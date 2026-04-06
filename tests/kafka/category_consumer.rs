#![allow(clippy::unwrap_used)]

use std::time::Duration;

use catalog::domain::category::category_id::CategoryId;
use catalog::domain::category::category_repository::ICategoryRepository;
use catalog::domain::shared::criteria::ScopedRepository;
use catalog::infrastructure::elasticsearch::category_es_repository::CategoryElasticSearchRepository;
use catalog::infrastructure::testing::es_helpers::EsTestHelper;
use catalog::infrastructure::testing::kafka_helpers::KafkaTestHelper;
use catalog::kafka::category_consumer::CategoryConsumer;
use catalog::kafka::consumer::build_consumer;
use uuid::Uuid;

fn random_group_id() -> String {
    format!("test_group_{}", Uuid::new_v4().simple())
}

fn random_topic() -> String {
    format!("test_prefix_{}.categories", Uuid::new_v4().simple())
}

async fn setup() -> (EsTestHelper, KafkaTestHelper, String) {
    let es_helper = EsTestHelper::start().await.expect("ES should start");
    let kafka_helper = KafkaTestHelper::start().await.expect("Kafka should start");
    let topic = random_topic();
    kafka_helper.create_topic(&topic).await.expect("Topic creation should succeed");
    (es_helper, kafka_helper, topic)
}

/// Runs the consumer in a background task for a short period to process pending messages.
async fn run_consumer_for(
    brokers: &str,
    topic: &str,
    es_helper: &EsTestHelper,
    duration: Duration,
) {
    let save_repo = CategoryElasticSearchRepository::new(es_helper.client(), es_helper.index());
    let delete_repo = CategoryElasticSearchRepository::new(es_helper.client(), es_helper.index());
    let category_consumer = CategoryConsumer::new(save_repo, delete_repo);
    let stream_consumer = build_consumer(brokers, &random_group_id())
        .expect("Consumer should build");

    tokio::time::timeout(
        duration,
        catalog::kafka::consumer::run_category_consumer(
            stream_consumer,
            topic,
            brokers,
            category_consumer,
            catalog::kafka::retry::RetryConfig::default(),
        ),
    )
    .await
    .ok(); // timeout is expected — we stop after duration
}

#[tokio::test]
async fn should_discard_tombstone_event() {
    let (es_helper, kafka_helper, topic) = setup().await;

    kafka_helper.send_tombstone(&topic).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    run_consumer_for(kafka_helper.brokers(), &topic, &es_helper, Duration::from_millis(800)).await;

    let repo = CategoryElasticSearchRepository::new(es_helper.client(), es_helper.index());
    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 0, "Tombstone should not create a category");
}

#[tokio::test]
async fn should_create_category_on_create_event() {
    let (es_helper, kafka_helper, topic) = setup().await;
    let category_id = Uuid::new_v4().to_string();

    let message = serde_json::json!({
        "op": "c",
        "before": null,
        "after": {
            "category_id": category_id,
            "name": "Movie",
            "description": "Movie category",
            "is_active": 1,
            "created_at": "2021-01-01T00:00:00Z"
        }
    });

    kafka_helper.send_json(&topic, &message.to_string()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    run_consumer_for(kafka_helper.brokers(), &topic, &es_helper, Duration::from_millis(800)).await;

    let repo = CategoryElasticSearchRepository::new(es_helper.client(), es_helper.index());
    let found = repo
        .find_by_id(&CategoryId::from(&category_id).unwrap())
        .await
        .unwrap();

    assert!(found.is_some(), "Category should have been created");
    let category = found.unwrap();
    assert_eq!(category.name(), "Movie");
    assert_eq!(category.description(), Some("Movie category"));
    assert!(category.is_active());
}

#[tokio::test]
async fn should_update_category_on_update_event() {
    let (es_helper, kafka_helper, topic) = setup().await;

    // First create
    let category_id = Uuid::new_v4().to_string();
    let create_msg = serde_json::json!({
        "op": "c",
        "before": null,
        "after": {
            "category_id": category_id,
            "name": "Original",
            "description": "original desc",
            "is_active": 1,
            "created_at": "2021-01-01T00:00:00Z"
        }
    });
    kafka_helper.send_json(&topic, &create_msg.to_string()).await.unwrap();

    // Then update
    let update_msg = serde_json::json!({
        "op": "u",
        "before": null,
        "after": {
            "category_id": category_id,
            "name": "Updated",
            "description": "updated desc",
            "is_active": 0,
            "created_at": "2021-01-01T00:00:00Z"
        }
    });
    kafka_helper.send_json(&topic, &update_msg.to_string()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    run_consumer_for(kafka_helper.brokers(), &topic, &es_helper, Duration::from_millis(1000)).await;

    let repo = CategoryElasticSearchRepository::new(es_helper.client(), es_helper.index());
    let found = repo
        .find_by_id(&CategoryId::from(&category_id).unwrap())
        .await
        .unwrap();

    assert!(found.is_some());
    let category = found.unwrap();
    assert_eq!(category.name(), "Updated");
    assert_eq!(category.description(), Some("updated desc"));
    assert!(!category.is_active());
}

#[tokio::test]
async fn should_soft_delete_category_on_delete_event() {
    let (es_helper, kafka_helper, topic) = setup().await;
    let category_id = Uuid::new_v4().to_string();

    // Create first
    let create_msg = serde_json::json!({
        "op": "c",
        "before": null,
        "after": {
            "category_id": category_id,
            "name": "ToDelete",
            "description": null,
            "is_active": 1,
            "created_at": "2021-01-01T00:00:00Z"
        }
    });
    kafka_helper.send_json(&topic, &create_msg.to_string()).await.unwrap();

    // Then delete
    let delete_msg = serde_json::json!({
        "op": "d",
        "before": {
            "category_id": category_id
        },
        "after": null
    });
    kafka_helper.send_json(&topic, &delete_msg.to_string()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    run_consumer_for(kafka_helper.brokers(), &topic, &es_helper, Duration::from_millis(1500)).await;

    let mut repo = CategoryElasticSearchRepository::new(es_helper.client(), es_helper.index());
    repo.ignore_soft_deleted();
    let found = repo
        .find_by_id(&CategoryId::from(&category_id).unwrap())
        .await
        .unwrap();

    assert!(found.is_none(), "Category should have been soft deleted");
}

#[tokio::test]
async fn should_discard_read_operation() {
    let (es_helper, kafka_helper, topic) = setup().await;

    let msg = serde_json::json!({
        "op": "r",
        "before": null,
        "after": null
    });
    kafka_helper.send_json(&topic, &msg.to_string()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    run_consumer_for(kafka_helper.brokers(), &topic, &es_helper, Duration::from_millis(800)).await;

    let repo = CategoryElasticSearchRepository::new(es_helper.client(), es_helper.index());
    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 0, "Read operation should not create a category");
}

#[tokio::test]
async fn should_skip_invalid_json_message() {
    let (es_helper, kafka_helper, topic) = setup().await;

    // Invalid JSON — consumer should log error and skip without crashing
    kafka_helper.send_json(&topic, "{ invalid json }").await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    run_consumer_for(kafka_helper.brokers(), &topic, &es_helper, Duration::from_millis(800)).await;

    let repo = CategoryElasticSearchRepository::new(es_helper.client(), es_helper.index());
    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 0);
}
