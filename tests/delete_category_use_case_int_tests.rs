#![allow(clippy::unwrap_used)]

use chrono::Utc;

use catalog::application::category::delete_category::{DeleteCategoryInput, DeleteCategoryUseCase};
use catalog::domain::category::aggregate::{Category, CategoryCreateCommand};
use catalog::domain::category::category_id::CategoryId;
use catalog::domain::category::category_repository::ICategoryRepository;
use catalog::infrastructure::elasticsearch::category_es_repository::CategoryElasticSearchRepository;
use catalog::infrastructure::testing::es_helpers::EsTestHelper;

#[tokio::test]
async fn should_soft_delete_category_in_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = CategoryElasticSearchRepository::new(helper.client.clone(), helper.index.clone());

    let category = Category::create(CategoryCreateCommand {
        category_id: CategoryId::new(),
        name: "Movie".to_string(),
        description: None,
        is_active: true,
        created_at: Utc::now(),
    });
    repo.insert(&category).await.unwrap();

    let repo2 = CategoryElasticSearchRepository::new(helper.client.clone(), helper.index.clone());
    let use_case = DeleteCategoryUseCase::new(repo2);
    use_case
        .execute(DeleteCategoryInput {
            id: category.category_id().to_string(),
        })
        .await
        .expect("should delete");

    let repo3 = CategoryElasticSearchRepository::new(helper.client.clone(), helper.index.clone());
    let found = repo3.find_by_id(category.category_id()).await.unwrap();
    assert!(found.is_some());
    assert!(found.unwrap().deleted_at().is_some());
}

#[tokio::test]
async fn should_error_when_deleting_not_found() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = CategoryElasticSearchRepository::new(helper.client.clone(), helper.index.clone());
    let use_case = DeleteCategoryUseCase::new(repo);

    let result = use_case
        .execute(DeleteCategoryInput {
            id: CategoryId::new().to_string(),
        })
        .await;

    assert!(result.is_err());
}
