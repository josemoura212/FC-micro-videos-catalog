#![allow(clippy::unwrap_used)]

use chrono::Utc;

use catalog::domain::category::category_id::CategoryId;
use catalog::domain::category::nested_category::NestedCategoryCreateCommand;
use catalog::domain::genre::aggregate::{Genre, GenreCreateCommand};
use catalog::domain::genre::genre_id::GenreId;
use catalog::domain::genre::genre_repository::IGenreRepository;
use catalog::domain::shared::criteria::ScopedRepository;
use catalog::infrastructure::elasticsearch::genre_es_repository::GenreElasticSearchRepository;
use catalog::infrastructure::testing::es_helpers::EsTestHelper;

fn make_nested_category_props() -> NestedCategoryCreateCommand {
    NestedCategoryCreateCommand {
        category_id: CategoryId::new(),
        name: "Movie".to_string(),
        is_active: true,
        deleted_at: None,
    }
}

fn make_genre(name: &str) -> Genre {
    Genre::create(GenreCreateCommand {
        genre_id: GenreId::new(),
        name: name.to_string(),
        categories_props: vec![make_nested_category_props()],
        is_active: true,
        created_at: Utc::now(),
    })
}

#[tokio::test]
async fn should_insert_and_find_by_id() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = GenreElasticSearchRepository::new(helper.client(), helper.index());
    let genre = make_genre("Action");

    repo.insert(&genre).await.unwrap();

    let found = repo.find_by_id(genre.genre_id()).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.name(), "Action");
    assert_eq!(found.categories().len(), 1);
}

#[tokio::test]
async fn should_return_none_when_not_found() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = GenreElasticSearchRepository::new(helper.client(), helper.index());
    let found = repo.find_by_id(&GenreId::new()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn should_bulk_insert() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = GenreElasticSearchRepository::new(helper.client(), helper.index());
    let genre1 = make_genre("Action");
    let genre2 = make_genre("Drama");

    repo.bulk_insert(&[genre1.clone(), genre2.clone()]).await.unwrap();

    let result = repo
        .find_by_ids(&[genre1.genre_id().clone(), genre2.genre_id().clone()])
        .await
        .unwrap();
    assert_eq!(result.exists.len(), 2);
}

#[tokio::test]
async fn should_update_genre() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = GenreElasticSearchRepository::new(helper.client(), helper.index());
    let mut genre = make_genre("Action");
    repo.insert(&genre).await.unwrap();

    genre.change_name("Drama".to_string());
    repo.update(&genre).await.unwrap();

    let found = repo.find_by_id(genre.genre_id()).await.unwrap().unwrap();
    assert_eq!(found.name(), "Drama");
}

#[tokio::test]
async fn should_delete_genre() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = GenreElasticSearchRepository::new(helper.client(), helper.index());
    let genre = make_genre("Action");
    repo.insert(&genre).await.unwrap();

    repo.delete(genre.genre_id()).await.unwrap();

    let found = repo.find_by_id(genre.genre_id()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn should_soft_delete_with_scope() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let mut repo = GenreElasticSearchRepository::new(helper.client(), helper.index());
    let mut genre = make_genre("Action");
    repo.insert(&genre).await.unwrap();

    genre.mark_as_deleted();
    repo.update(&genre).await.unwrap();

    repo.clear_scopes();
    let found = repo.find_by_id(genre.genre_id()).await.unwrap();
    assert!(found.is_some());

    repo.ignore_soft_deleted();
    let found = repo.find_by_id(genre.genre_id()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn should_find_all() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = GenreElasticSearchRepository::new(helper.client(), helper.index());
    let genre1 = make_genre("Action");
    let genre2 = make_genre("Drama");
    repo.insert(&genre1).await.unwrap();
    repo.insert(&genre2).await.unwrap();

    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn should_exists_by_id() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = GenreElasticSearchRepository::new(helper.client(), helper.index());
    let genre = make_genre("Action");
    repo.insert(&genre).await.unwrap();

    let result = repo.exists_by_id(&[genre.genre_id().clone()]).await.unwrap();
    assert_eq!(result.exists.len(), 1);
    assert!(result.not_exists.is_empty());

    let fake_id = GenreId::new();
    let result = repo.exists_by_id(&[fake_id]).await.unwrap();
    assert!(result.exists.is_empty());
    assert_eq!(result.not_exists.len(), 1);
}
