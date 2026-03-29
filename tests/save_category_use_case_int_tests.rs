#![allow(clippy::unwrap_used)]

use chrono::Utc;

use catalog::application::category::save_category::{SaveCategoryInput, SaveCategoryUseCase};
use catalog::domain::category::category_id::CategoryId;
use catalog::domain::category::category_repository::ICategoryRepository;
use catalog::infrastructure::elasticsearch::category_es_repository::CategoryElasticSearchRepository;
use catalog::infrastructure::testing::es_helpers::EsTestHelper;

#[tokio::test]
async fn should_create_category_in_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = CategoryElasticSearchRepository::new(helper.client.clone(), helper.index.clone());
    let use_case = SaveCategoryUseCase::new(repo);

    let category_id = CategoryId::new();
    let created_at = Utc::now();

    let output = use_case
        .execute(SaveCategoryInput {
            category_id: category_id.to_string(),
            name: "Movie".to_string(),
            description: Some("some description".to_string()),
            is_active: false,
            created_at,
        })
        .await
        .expect("should create");

    assert_eq!(output.id, category_id.to_string());
    assert!(output.created);

    let repo2 = CategoryElasticSearchRepository::new(helper.client.clone(), helper.index.clone());
    let found = repo2.find_by_id(&category_id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.name(), "Movie");
    assert!(!found.is_active());
}

#[tokio::test]
async fn should_update_category_in_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = CategoryElasticSearchRepository::new(helper.client.clone(), helper.index.clone());

    let category_id = CategoryId::new();
    let use_case = SaveCategoryUseCase::new(repo);

    use_case
        .execute(SaveCategoryInput {
            category_id: category_id.to_string(),
            name: "Movie".to_string(),
            description: None,
            is_active: true,
            created_at: Utc::now(),
        })
        .await
        .expect("should create");

    let repo2 = CategoryElasticSearchRepository::new(helper.client.clone(), helper.index.clone());
    let use_case2 = SaveCategoryUseCase::new(repo2);

    let output = use_case2
        .execute(SaveCategoryInput {
            category_id: category_id.to_string(),
            name: "Documentary".to_string(),
            description: Some("updated".to_string()),
            is_active: false,
            created_at: Utc::now(),
        })
        .await
        .expect("should update");

    assert_eq!(output.id, category_id.to_string());
    assert!(!output.created);
}
