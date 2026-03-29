#![allow(clippy::unwrap_used)]

use chrono::Utc;

use catalog::application::category::get_category::{GetCategoryInput, GetCategoryUseCase};
use catalog::domain::category::aggregate::{Category, CategoryCreateCommand};
use catalog::domain::category::category_id::CategoryId;
use catalog::domain::category::category_repository::ICategoryRepository;
use catalog::infrastructure::elasticsearch::category_es_repository::CategoryElasticSearchRepository;
use catalog::infrastructure::testing::es_helpers::EsTestHelper;

#[tokio::test]
async fn should_get_category_from_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = CategoryElasticSearchRepository::new(helper.client.clone(), helper.index.clone());

    let category = Category::create(CategoryCreateCommand {
        category_id: CategoryId::new(),
        name: "Movie".to_string(),
        description: Some("desc".to_string()),
        is_active: true,
        created_at: Utc::now(),
    });
    repo.insert(&category).await.unwrap();

    let repo2 = CategoryElasticSearchRepository::new(helper.client.clone(), helper.index.clone());
    let use_case = GetCategoryUseCase::new(repo2);

    let output = use_case
        .execute(GetCategoryInput {
            id: category.category_id().to_string(),
        })
        .await
        .expect("should find");

    assert_eq!(output.id, category.category_id().to_string());
    assert_eq!(output.name, "Movie");
}

#[tokio::test]
async fn should_error_when_not_found_in_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = CategoryElasticSearchRepository::new(helper.client.clone(), helper.index.clone());
    let use_case = GetCategoryUseCase::new(repo);

    let result = use_case
        .execute(GetCategoryInput {
            id: CategoryId::new().to_string(),
        })
        .await;

    assert!(result.is_err());
}
