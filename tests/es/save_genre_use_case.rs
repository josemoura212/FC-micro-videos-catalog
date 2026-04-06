#![allow(clippy::unwrap_used)]

use chrono::Utc;

use catalog::application::genre::save_genre::{SaveGenreInput, SaveGenreUseCase};
use catalog::domain::category::aggregate::{Category, CategoryCreateCommand};
use catalog::domain::category::category_id::CategoryId;
use catalog::domain::category::category_repository::ICategoryRepository;
use catalog::domain::category::nested_category::NestedCategoryCreateCommand;
use catalog::domain::genre::aggregate::{Genre, GenreCreateCommand};
use catalog::domain::genre::genre_id::GenreId;
use catalog::domain::genre::genre_repository::IGenreRepository;
use catalog::infrastructure::elasticsearch::category_es_repository::CategoryElasticSearchRepository;
use catalog::infrastructure::elasticsearch::genre_es_repository::GenreElasticSearchRepository;
use catalog::infrastructure::testing::es_helpers::EsTestHelper;

#[tokio::test]
async fn should_create_genre_in_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let genre_repo = GenreElasticSearchRepository::new(helper.client(), helper.index());
    let category_repo = CategoryElasticSearchRepository::new(helper.client(), helper.index());

    let category = Category::create(CategoryCreateCommand {
        category_id: CategoryId::new(),
        name: "Movie".to_string(),
        description: Some("some description".to_string()),
        is_active: true,
        created_at: Utc::now(),
    });
    category_repo.insert(&category).await.unwrap();

    let use_case = SaveGenreUseCase::new(genre_repo, category_repo);
    let genre_id = GenreId::new();

    let output = use_case
        .execute(SaveGenreInput {
            genre_id: genre_id.to_string(),
            name: "Action".to_string(),
            categories_id: vec![category.category_id().to_string()],
            is_active: true,
            created_at: Utc::now(),
        })
        .await
        .expect("should create");

    assert_eq!(output.id, genre_id.to_string());
    assert!(output.created);
}

#[tokio::test]
async fn should_update_genre_in_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let genre_repo = GenreElasticSearchRepository::new(helper.client(), helper.index());
    let category_repo = CategoryElasticSearchRepository::new(helper.client(), helper.index());

    let category = Category::create(CategoryCreateCommand {
        category_id: CategoryId::new(),
        name: "Movie".to_string(),
        description: None,
        is_active: true,
        created_at: Utc::now(),
    });
    category_repo.insert(&category).await.unwrap();

    let genre = Genre::create(GenreCreateCommand {
        genre_id: GenreId::new(),
        name: "Action".to_string(),
        categories_props: vec![NestedCategoryCreateCommand {
            category_id: category.category_id().clone(),
            name: "Movie".to_string(),
            is_active: true,
            deleted_at: None,
        }],
        is_active: true,
        created_at: Utc::now(),
    });
    genre_repo.insert(&genre).await.unwrap();

    let use_case = SaveGenreUseCase::new(genre_repo, category_repo);

    let output = use_case
        .execute(SaveGenreInput {
            genre_id: genre.genre_id().to_string(),
            name: "Drama".to_string(),
            categories_id: vec![category.category_id().to_string()],
            is_active: false,
            created_at: Utc::now(),
        })
        .await
        .expect("should update");

    assert_eq!(output.id, genre.genre_id().to_string());
    assert!(!output.created);
}
