use chrono::{DateTime, Utc};

use crate::domain::category::aggregate::{Category, CategoryCreateCommand};
use crate::domain::category::category_id::CategoryId;
use crate::domain::category::category_repository::ICategoryRepository;
use crate::domain::shared::entity::Entity;
use crate::domain::shared::errors::EntityValidationError;

#[derive(Debug, Clone)]
pub struct SaveCategoryInput {
    pub category_id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SaveCategoryOutput {
    pub id: String,
    pub created: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum SaveCategoryError<E: std::error::Error> {
    #[error(transparent)]
    Validation(#[from] EntityValidationError),
    #[error("invalid category id: {0}")]
    InvalidId(String),
    #[error(transparent)]
    Repository(E),
}

pub struct SaveCategoryUseCase<R: ICategoryRepository> {
    repo: R,
}

impl<R: ICategoryRepository> SaveCategoryUseCase<R> {
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// # Errors
    /// Returns error on validation failure or repository error.
    pub async fn execute(
        &self,
        input: SaveCategoryInput,
    ) -> Result<SaveCategoryOutput, SaveCategoryError<R::Error>> {
        let category_id = CategoryId::from(&input.category_id)
            .map_err(|e| SaveCategoryError::InvalidId(e.to_string()))?;

        let existing = self
            .repo
            .find_by_id(&category_id)
            .await
            .map_err(SaveCategoryError::Repository)?;

        match existing {
            Some(category) => self.update_category(input, category).await,
            None => self.create_category(input, category_id).await,
        }
    }

    async fn create_category(
        &self,
        input: SaveCategoryInput,
        category_id: CategoryId,
    ) -> Result<SaveCategoryOutput, SaveCategoryError<R::Error>> {
        let entity = Category::create(CategoryCreateCommand {
            category_id,
            name: input.name,
            description: input.description,
            is_active: input.is_active,
            created_at: input.created_at,
        });

        if entity.notification().has_errors() {
            return Err(EntityValidationError::new(entity.notification().clone()).into());
        }

        self.repo
            .insert(&entity)
            .await
            .map_err(SaveCategoryError::Repository)?;

        Ok(SaveCategoryOutput {
            id: entity.category_id().to_string(),
            created: true,
        })
    }

    async fn update_category(
        &self,
        input: SaveCategoryInput,
        mut category: Category,
    ) -> Result<SaveCategoryOutput, SaveCategoryError<R::Error>> {
        category.change_name(input.name);
        category.change_description(input.description);

        if input.is_active {
            category.activate();
        } else {
            category.deactivate();
        }

        category.change_created_at(input.created_at);

        if category.notification().has_errors() {
            return Err(EntityValidationError::new(category.notification().clone()).into());
        }

        self.repo
            .update(&category)
            .await
            .map_err(SaveCategoryError::Repository)?;

        Ok(SaveCategoryOutput {
            id: category.category_id().to_string(),
            created: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::in_memory::category_in_memory_repository::CategoryInMemoryRepository;

    #[tokio::test]
    async fn should_create_category() {
        let repo = CategoryInMemoryRepository::new();
        let use_case = SaveCategoryUseCase::new(repo);
        let category_id = CategoryId::new();

        let output = use_case
            .execute(SaveCategoryInput {
                category_id: category_id.to_string(),
                name: "Movie".to_string(),
                description: Some("some description".to_string()),
                is_active: true,
                created_at: Utc::now(),
            })
            .await
            .expect("should create");

        assert_eq!(output.id, category_id.to_string());
        assert!(output.created);
    }

    #[tokio::test]
    async fn should_update_category() {
        let repo = CategoryInMemoryRepository::new();
        let category = Category::create(CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            description: None,
            is_active: true,
            created_at: Utc::now(),
        });
        repo.insert(&category).await.expect("insert");

        let use_case = SaveCategoryUseCase::new(repo);
        let output = use_case
            .execute(SaveCategoryInput {
                category_id: category.category_id().to_string(),
                name: "Documentary".to_string(),
                description: Some("updated".to_string()),
                is_active: false,
                created_at: Utc::now(),
            })
            .await
            .expect("should update");

        assert_eq!(output.id, category.category_id().to_string());
        assert!(!output.created);
    }

    #[tokio::test]
    async fn should_fail_with_invalid_name() {
        let repo = CategoryInMemoryRepository::new();
        let use_case = SaveCategoryUseCase::new(repo);

        let result = use_case
            .execute(SaveCategoryInput {
                category_id: CategoryId::new().to_string(),
                name: "a".repeat(256),
                description: None,
                is_active: true,
                created_at: Utc::now(),
            })
            .await;

        assert!(result.is_err());
    }
}
