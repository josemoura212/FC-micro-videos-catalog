#![allow(clippy::unwrap_used)]

use chrono::Utc;

use catalog::application::category::list_all_categories::ListAllCategoriesUseCase;
use catalog::domain::category::aggregate::{Category, CategoryCreateCommand};
use catalog::domain::category::category_id::CategoryId;
use catalog::domain::category::category_repository::ICategoryRepository;
use catalog::domain::shared::criteria::ScopedRepository;
use catalog::infrastructure::elasticsearch::category_es_repository::CategoryElasticSearchRepository;
use catalog::infrastructure::testing::es_helpers::EsTestHelper;

#[tokio::test]
async fn should_list_all_categories_from_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = CategoryElasticSearchRepository::new(helper.client.clone(), helper.index.clone());

    let cat1 = Category::create(CategoryCreateCommand {
        category_id: CategoryId::new(),
        name: "Movie".to_string(),
        description: None,
        is_active: true,
        created_at: Utc::now(),
    });
    let cat2 = Category::create(CategoryCreateCommand {
        category_id: CategoryId::new(),
        name: "Documentary".to_string(),
        description: None,
        is_active: true,
        created_at: Utc::now(),
    });
    repo.insert(&cat1).await.unwrap();
    repo.insert(&cat2).await.unwrap();

    let repo2 = CategoryElasticSearchRepository::new(helper.client.clone(), helper.index.clone());
    let use_case = ListAllCategoriesUseCase::new(repo2);
    let output = use_case.execute().await.expect("should list");

    assert_eq!(output.len(), 2);
}

#[tokio::test]
async fn should_exclude_soft_deleted_from_list() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = CategoryElasticSearchRepository::new(helper.client.clone(), helper.index.clone());

    let cat1 = Category::create(CategoryCreateCommand {
        category_id: CategoryId::new(),
        name: "Movie".to_string(),
        description: None,
        is_active: true,
        created_at: Utc::now(),
    });
    let mut cat2 = Category::create(CategoryCreateCommand {
        category_id: CategoryId::new(),
        name: "Documentary".to_string(),
        description: None,
        is_active: true,
        created_at: Utc::now(),
    });
    repo.insert(&cat1).await.unwrap();
    repo.insert(&cat2).await.unwrap();

    cat2.mark_as_deleted();
    repo.update(&cat2).await.unwrap();

    let mut repo2 =
        CategoryElasticSearchRepository::new(helper.client.clone(), helper.index.clone());
    repo2.ignore_soft_deleted();
    let use_case = ListAllCategoriesUseCase::new(repo2);
    let output = use_case.execute().await.expect("should list");

    assert_eq!(output.len(), 1);
    assert_eq!(output[0].name, "Movie");
}
