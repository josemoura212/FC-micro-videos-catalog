#![allow(clippy::unwrap_used)]

use chrono::Utc;

use catalog::application::category::save_category::{SaveCategoryInput, SaveCategoryUseCase};
use catalog::domain::category::category_id::CategoryId;
use catalog::infrastructure::elasticsearch::category_es_repository::CategoryElasticSearchRepository;
use catalog::infrastructure::testing::es_helpers::EsTestHelper;

#[tokio::test]
async fn should_create_category_in_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = CategoryElasticSearchRepository::new(helper.client, helper.index);
    let use_case = SaveCategoryUseCase::new(repo);

    let category_id = CategoryId::new();
    let output = use_case
        .execute(SaveCategoryInput {
            category_id: category_id.to_string(),
            name: "Movie".to_string(),
            description: Some("some description".to_string()),
            is_active: false,
            created_at: Utc::now(),
        })
        .await
        .expect("should create");

    assert_eq!(output.id, category_id.to_string());
    assert!(output.created);
}

#[tokio::test]
async fn should_update_category_in_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = CategoryElasticSearchRepository::new(helper.client, helper.index);
    let use_case = SaveCategoryUseCase::new(repo);

    let category_id = CategoryId::new();
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

    let output = use_case
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
