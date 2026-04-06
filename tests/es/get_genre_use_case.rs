#![allow(clippy::unwrap_used)]

use chrono::Utc;

use catalog::application::genre::get_genre::{GetGenreInput, GetGenreUseCase};
use catalog::domain::category::category_id::CategoryId;
use catalog::domain::category::nested_category::NestedCategoryCreateCommand;
use catalog::domain::genre::aggregate::{Genre, GenreCreateCommand};
use catalog::domain::genre::genre_id::GenreId;
use catalog::domain::genre::genre_repository::IGenreRepository;
use catalog::infrastructure::elasticsearch::genre_es_repository::GenreElasticSearchRepository;
use catalog::infrastructure::testing::es_helpers::EsTestHelper;

#[tokio::test]
async fn should_get_genre_from_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = GenreElasticSearchRepository::new(helper.client(), helper.index());

    let cat_id = CategoryId::new();
    let genre = Genre::create(GenreCreateCommand {
        genre_id: GenreId::new(),
        name: "Action".to_string(),
        categories_props: vec![NestedCategoryCreateCommand {
            category_id: cat_id.clone(),
            name: "Movie".to_string(),
            is_active: true,
            deleted_at: None,
        }],
        is_active: true,
        created_at: Utc::now(),
    });
    repo.insert(&genre).await.unwrap();

    let use_case = GetGenreUseCase::new(repo);
    let output = use_case
        .execute(GetGenreInput {
            id: genre.genre_id().to_string(),
        })
        .await
        .expect("should find");

    assert_eq!(output.id, genre.genre_id().to_string());
    assert_eq!(output.name, "Action");
    assert!(output.is_active);
    assert_eq!(output.categories.len(), 1);
    assert_eq!(output.categories[0].id, cat_id.to_string());
}

#[tokio::test]
async fn should_error_when_not_found_in_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = GenreElasticSearchRepository::new(helper.client(), helper.index());
    let use_case = GetGenreUseCase::new(repo);

    let result = use_case
        .execute(GetGenreInput {
            id: GenreId::new().to_string(),
        })
        .await;

    assert!(result.is_err());
}
