#![allow(clippy::unwrap_used)]

use chrono::Utc;

use catalog::application::genre::delete_genre::{DeleteGenreInput, DeleteGenreUseCase};
use catalog::domain::category::category_id::CategoryId;
use catalog::domain::category::nested_category::NestedCategoryCreateCommand;
use catalog::domain::genre::aggregate::{Genre, GenreCreateCommand};
use catalog::domain::genre::genre_id::GenreId;
use catalog::domain::genre::genre_repository::IGenreRepository;
use catalog::infrastructure::elasticsearch::genre_es_repository::GenreElasticSearchRepository;
use catalog::infrastructure::testing::es_helpers::EsTestHelper;

#[tokio::test]
async fn should_soft_delete_genre_in_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = GenreElasticSearchRepository::new(helper.client(), helper.index());

    let genre = Genre::create(GenreCreateCommand {
        genre_id: GenreId::new(),
        name: "Action".to_string(),
        categories_props: vec![NestedCategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            is_active: true,
            deleted_at: None,
        }],
        is_active: true,
        created_at: Utc::now(),
    });
    repo.insert(&genre).await.unwrap();

    let use_case = DeleteGenreUseCase::new(repo);
    use_case
        .execute(DeleteGenreInput {
            id: genre.genre_id().to_string(),
        })
        .await
        .expect("should delete");
}

#[tokio::test]
async fn should_error_when_deleting_not_found() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = GenreElasticSearchRepository::new(helper.client(), helper.index());
    let use_case = DeleteGenreUseCase::new(repo);

    let result = use_case
        .execute(DeleteGenreInput {
            id: GenreId::new().to_string(),
        })
        .await;

    assert!(result.is_err());
}
