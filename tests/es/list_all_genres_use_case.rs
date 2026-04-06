#![allow(clippy::unwrap_used)]

use chrono::Utc;

use catalog::application::genre::list_all_genres::ListAllGenresUseCase;
use catalog::domain::category::category_id::CategoryId;
use catalog::domain::category::nested_category::NestedCategoryCreateCommand;
use catalog::domain::genre::aggregate::{Genre, GenreCreateCommand};
use catalog::domain::genre::genre_id::GenreId;
use catalog::domain::genre::genre_repository::IGenreRepository;
use catalog::domain::shared::criteria::ScopedRepository;
use catalog::infrastructure::elasticsearch::genre_es_repository::GenreElasticSearchRepository;
use catalog::infrastructure::testing::es_helpers::EsTestHelper;

#[tokio::test]
async fn should_list_all_genres_from_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = GenreElasticSearchRepository::new(helper.client(), helper.index());

    let genre1 = Genre::create(GenreCreateCommand {
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
    let genre2 = Genre::create(GenreCreateCommand {
        genre_id: GenreId::new(),
        name: "Drama".to_string(),
        categories_props: vec![],
        is_active: true,
        created_at: Utc::now(),
    });
    repo.insert(&genre1).await.unwrap();
    repo.insert(&genre2).await.unwrap();

    let use_case = ListAllGenresUseCase::new(repo);
    let output = use_case.execute().await.expect("should list");

    assert_eq!(output.len(), 2);
}

#[tokio::test]
async fn should_exclude_soft_deleted_from_list() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let mut repo = GenreElasticSearchRepository::new(helper.client(), helper.index());

    let genre1 = Genre::create(GenreCreateCommand {
        genre_id: GenreId::new(),
        name: "Action".to_string(),
        categories_props: vec![],
        is_active: true,
        created_at: Utc::now(),
    });
    let mut genre2 = Genre::create(GenreCreateCommand {
        genre_id: GenreId::new(),
        name: "Drama".to_string(),
        categories_props: vec![],
        is_active: true,
        created_at: Utc::now(),
    });
    repo.insert(&genre1).await.unwrap();
    repo.insert(&genre2).await.unwrap();

    genre2.mark_as_deleted();
    repo.update(&genre2).await.unwrap();

    repo.ignore_soft_deleted();
    let use_case = ListAllGenresUseCase::new(repo);
    let output = use_case.execute().await.expect("should list");

    assert_eq!(output.len(), 1);
    assert_eq!(output[0].name, "Action");
}
