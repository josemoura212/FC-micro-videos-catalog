#![allow(clippy::unwrap_used)]

use chrono::Utc;

use catalog::domain::category::aggregate::{Category, CategoryCreateCommand};
use catalog::domain::category::category_id::CategoryId;
use catalog::domain::category::category_repository::ICategoryRepository;
use catalog::domain::shared::criteria::ScopedRepository;
use catalog::infrastructure::elasticsearch::category_es_repository::CategoryElasticSearchRepository;
use catalog::infrastructure::testing::es_helpers::EsTestHelper;

fn make_category(name: &str) -> Category {
    Category::create(CategoryCreateCommand {
        category_id: CategoryId::new(),
        name: name.to_string(),
        description: Some("some description".to_string()),
        is_active: true,
        created_at: Utc::now(),
    })
}

#[tokio::test]
async fn should_insert_and_find_by_id() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CategoryElasticSearchRepository::new(helper.client, helper.index);
    let category = make_category("Movie");

    repo.insert(&category).await.unwrap();

    let found = repo.find_by_id(category.category_id()).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.name(), "Movie");
    assert_eq!(found.description(), Some("some description"));
}

#[tokio::test]
async fn should_return_none_when_not_found() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CategoryElasticSearchRepository::new(helper.client, helper.index);
    let found = repo.find_by_id(&CategoryId::new()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn should_bulk_insert() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CategoryElasticSearchRepository::new(helper.client, helper.index);
    let cat1 = make_category("Movie");
    let cat2 = make_category("Documentary");

    repo.bulk_insert(&[cat1.clone(), cat2.clone()]).await.unwrap();

    let result = repo
        .find_by_ids(&[cat1.category_id().clone(), cat2.category_id().clone()])
        .await
        .unwrap();
    assert_eq!(result.exists.len(), 2);
}

#[tokio::test]
async fn should_find_one_by_filter() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CategoryElasticSearchRepository::new(helper.client, helper.index);
    let category = make_category("Movie");
    repo.insert(&category).await.unwrap();

    let found = repo
        .find_one_by(Some(category.category_id()), Some(true))
        .await
        .unwrap();
    assert!(found.is_some());

    let not_found = repo
        .find_one_by(Some(category.category_id()), Some(false))
        .await
        .unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn should_find_by_with_sort() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CategoryElasticSearchRepository::new(helper.client, helper.index);
    let cat_a = make_category("AAA");
    let cat_b = make_category("BBB");
    repo.insert(&cat_b).await.unwrap();
    repo.insert(&cat_a).await.unwrap();

    let order = catalog::domain::shared::repository::SortOrder {
        field: "name".to_string(),
        direction: catalog::domain::shared::repository::SortDirection::Asc,
    };

    let results = repo.find_by(None, Some(true), Some(&order)).await.unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].name(), "AAA");
    assert_eq!(results[1].name(), "BBB");
}

#[tokio::test]
async fn should_update_category() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CategoryElasticSearchRepository::new(helper.client, helper.index);
    let mut category = make_category("Movie");
    repo.insert(&category).await.unwrap();

    category.change_name("Documentary".to_string());
    repo.update(&category).await.unwrap();

    let found = repo.find_by_id(category.category_id()).await.unwrap().unwrap();
    assert_eq!(found.name(), "Documentary");
}

#[tokio::test]
async fn should_error_on_update_not_found() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CategoryElasticSearchRepository::new(helper.client, helper.index);
    let category = make_category("Movie");
    let result = repo.update(&category).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn should_delete_category() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CategoryElasticSearchRepository::new(helper.client, helper.index);
    let category = make_category("Movie");
    repo.insert(&category).await.unwrap();

    repo.delete(category.category_id()).await.unwrap();

    let found = repo.find_by_id(category.category_id()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn should_soft_delete_with_scope() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let mut repo = CategoryElasticSearchRepository::new(helper.client, helper.index);
    let mut category = make_category("Movie");
    repo.insert(&category).await.unwrap();

    category.mark_as_deleted();
    repo.update(&category).await.unwrap();

    repo.clear_scopes();
    let found = repo.find_by_id(category.category_id()).await.unwrap();
    assert!(found.is_some());

    repo.ignore_soft_deleted();
    let found = repo.find_by_id(category.category_id()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn should_find_all() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CategoryElasticSearchRepository::new(helper.client, helper.index);
    let cat1 = make_category("Movie");
    let cat2 = make_category("Documentary");
    repo.insert(&cat1).await.unwrap();
    repo.insert(&cat2).await.unwrap();

    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn should_exists_by_id() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CategoryElasticSearchRepository::new(helper.client, helper.index);
    let category = make_category("Movie");
    repo.insert(&category).await.unwrap();

    let result = repo.exists_by_id(&[category.category_id().clone()]).await.unwrap();
    assert_eq!(result.exists.len(), 1);
    assert!(result.not_exists.is_empty());

    let fake_id = CategoryId::new();
    let result = repo.exists_by_id(&[fake_id]).await.unwrap();
    assert!(result.exists.is_empty());
    assert_eq!(result.not_exists.len(), 1);
}
