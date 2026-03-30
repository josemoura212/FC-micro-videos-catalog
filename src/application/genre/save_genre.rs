use chrono::{DateTime, Utc};

use crate::domain::category::category_id::CategoryId;
use crate::domain::category::category_repository::ICategoryRepository;
use crate::domain::category::nested_category::NestedCategoryCreateCommand;
use crate::domain::genre::aggregate::{Genre, GenreCreateCommand};
use crate::domain::genre::genre_id::GenreId;
use crate::domain::genre::genre_repository::IGenreRepository;
use crate::domain::shared::entity::Entity;
use crate::domain::shared::errors::EntityValidationError;

#[derive(Debug, Clone)]
pub struct SaveGenreInput {
    pub genre_id: String,
    pub name: String,
    pub categories_id: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SaveGenreOutput {
    pub id: String,
    pub created: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum SaveGenreError<GE: std::error::Error, CE: std::error::Error> {
    #[error(transparent)]
    Validation(#[from] EntityValidationError),
    #[error("invalid genre id: {0}")]
    InvalidId(String),
    #[error("categories not found: {0:?}")]
    CategoriesNotFound(Vec<String>),
    #[error(transparent)]
    GenreRepository(GE),
    #[error("category repository error: {0}")]
    CategoryRepository(CE),
}

pub struct SaveGenreUseCase<GR: IGenreRepository, CR: ICategoryRepository> {
    genre_repo: GR,
    category_repo: CR,
}

impl<GR: IGenreRepository, CR: ICategoryRepository> SaveGenreUseCase<GR, CR> {
    #[must_use]
    pub const fn new(genre_repo: GR, category_repo: CR) -> Self {
        Self {
            genre_repo,
            category_repo,
        }
    }

    /// # Errors
    /// Returns error on validation failure, categories not found, or repository error.
    pub async fn execute(
        &self,
        input: SaveGenreInput,
    ) -> Result<SaveGenreOutput, SaveGenreError<GR::Error, CR::Error>> {
        let genre_id = GenreId::from(&input.genre_id)
            .map_err(|e| SaveGenreError::InvalidId(e.to_string()))?;

        let existing = self
            .genre_repo
            .find_by_id(&genre_id)
            .await
            .map_err(SaveGenreError::GenreRepository)?;

        match existing {
            Some(genre) => self.update_genre(input, genre).await,
            None => self.create_genre(input, genre_id).await,
        }
    }

    async fn create_genre(
        &self,
        input: SaveGenreInput,
        genre_id: GenreId,
    ) -> Result<SaveGenreOutput, SaveGenreError<GR::Error, CR::Error>> {
        let categories_props = self.get_categories_props(&input.categories_id).await?;

        let entity = Genre::create(GenreCreateCommand {
            genre_id,
            name: input.name,
            categories_props,
            is_active: input.is_active,
            created_at: input.created_at,
        });

        if entity.notification().has_errors() {
            return Err(EntityValidationError::new(entity.notification().clone()).into());
        }

        self.genre_repo
            .insert(&entity)
            .await
            .map_err(SaveGenreError::GenreRepository)?;

        Ok(SaveGenreOutput {
            id: entity.genre_id().to_string(),
            created: true,
        })
    }

    async fn update_genre(
        &self,
        input: SaveGenreInput,
        mut genre: Genre,
    ) -> Result<SaveGenreOutput, SaveGenreError<GR::Error, CR::Error>> {
        let categories_props = self.get_categories_props(&input.categories_id).await?;

        genre.change_name(input.name);

        let nested_categories = categories_props
            .into_iter()
            .map(|props| {
                crate::domain::category::nested_category::NestedCategory::create(props)
            })
            .collect();
        genre.sync_nested_categories(nested_categories);

        if input.is_active {
            genre.activate();
        } else {
            genre.deactivate();
        }

        genre.change_created_at(input.created_at);

        if genre.notification().has_errors() {
            return Err(EntityValidationError::new(genre.notification().clone()).into());
        }

        self.genre_repo
            .update(&genre)
            .await
            .map_err(SaveGenreError::GenreRepository)?;

        Ok(SaveGenreOutput {
            id: genre.genre_id().to_string(),
            created: false,
        })
    }

    async fn get_categories_props(
        &self,
        categories_id: &[String],
    ) -> Result<Vec<NestedCategoryCreateCommand>, SaveGenreError<GR::Error, CR::Error>> {
        let category_ids: Vec<CategoryId> = categories_id
            .iter()
            .map(|id| {
                CategoryId::from(id)
                    .map_err(|e| SaveGenreError::InvalidId(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let result = self
            .category_repo
            .find_by_ids(&category_ids)
            .await
            .map_err(SaveGenreError::CategoryRepository)?;

        if !result.not_exists.is_empty() {
            let not_found_ids: Vec<String> =
                result.not_exists.iter().map(ToString::to_string).collect();
            return Err(SaveGenreError::CategoriesNotFound(not_found_ids));
        }

        Ok(result
            .exists
            .iter()
            .map(|category| NestedCategoryCreateCommand {
                category_id: category.category_id().clone(),
                name: category.name().to_string(),
                is_active: category.is_active(),
                deleted_at: category.deleted_at(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::category::aggregate::{Category, CategoryCreateCommand};
    use crate::infrastructure::in_memory::category_in_memory_repository::CategoryInMemoryRepository;
    use crate::infrastructure::in_memory::genre_in_memory_repository::GenreInMemoryRepository;

    #[tokio::test]
    async fn should_create_genre() {
        let genre_repo = GenreInMemoryRepository::new();
        let category_repo = CategoryInMemoryRepository::new();

        let category = Category::create(CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            description: Some("some description".to_string()),
            is_active: true,
            created_at: Utc::now(),
        });
        category_repo.insert(&category).await.expect("insert category");

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
    async fn should_update_genre() {
        let genre_repo = GenreInMemoryRepository::new();
        let category_repo = CategoryInMemoryRepository::new();

        let category = Category::create(CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            description: None,
            is_active: true,
            created_at: Utc::now(),
        });
        category_repo.insert(&category).await.expect("insert category");

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
        genre_repo.insert(&genre).await.expect("insert genre");

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

    #[tokio::test]
    async fn should_fail_with_invalid_name() {
        let genre_repo = GenreInMemoryRepository::new();
        let category_repo = CategoryInMemoryRepository::new();

        let category = Category::create(CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            description: None,
            is_active: true,
            created_at: Utc::now(),
        });
        category_repo.insert(&category).await.expect("insert category");

        let use_case = SaveGenreUseCase::new(genre_repo, category_repo);

        let result = use_case
            .execute(SaveGenreInput {
                genre_id: GenreId::new().to_string(),
                name: "a".repeat(256),
                categories_id: vec![category.category_id().to_string()],
                is_active: true,
                created_at: Utc::now(),
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_fail_when_categories_not_found() {
        let genre_repo = GenreInMemoryRepository::new();
        let category_repo = CategoryInMemoryRepository::new();

        let use_case = SaveGenreUseCase::new(genre_repo, category_repo);

        let result = use_case
            .execute(SaveGenreInput {
                genre_id: GenreId::new().to_string(),
                name: "Action".to_string(),
                categories_id: vec![CategoryId::new().to_string()],
                is_active: true,
                created_at: Utc::now(),
            })
            .await;

        assert!(matches!(result, Err(SaveGenreError::CategoriesNotFound(_))));
    }
}
