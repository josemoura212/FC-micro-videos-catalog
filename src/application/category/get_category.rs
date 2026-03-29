use crate::domain::category::category_id::CategoryId;
use crate::domain::category::category_repository::ICategoryRepository;
use crate::domain::shared::errors::NotFoundError;

use super::category_output::{CategoryOutput, CategoryOutputMapper};

#[derive(Debug, Clone)]
pub struct GetCategoryInput {
    pub id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GetCategoryError<E: std::error::Error> {
    #[error(transparent)]
    NotFound(#[from] NotFoundError),
    #[error("invalid category id: {0}")]
    InvalidId(String),
    #[error(transparent)]
    Repository(E),
}

pub struct GetCategoryUseCase<R: ICategoryRepository> {
    repo: R,
}

impl<R: ICategoryRepository> GetCategoryUseCase<R> {
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// # Errors
    /// Returns `NotFoundError` if category not found, or repository error.
    pub async fn execute(
        &self,
        input: GetCategoryInput,
    ) -> Result<CategoryOutput, GetCategoryError<R::Error>> {
        let category_id = CategoryId::from(&input.id)
            .map_err(|e| GetCategoryError::InvalidId(e.to_string()))?;

        let category = self
            .repo
            .find_one_by(Some(&category_id), Some(true))
            .await
            .map_err(GetCategoryError::Repository)?
            .ok_or_else(|| NotFoundError::new(&input.id, "Category"))?;

        Ok(CategoryOutputMapper::to_output(&category))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::category::aggregate::{Category, CategoryCreateCommand};
    use crate::infrastructure::in_memory::category_in_memory_repository::CategoryInMemoryRepository;

    #[tokio::test]
    async fn should_return_category() {
        let repo = CategoryInMemoryRepository::new();
        let category = Category::create(CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            description: Some("desc".to_string()),
            is_active: true,
            created_at: Utc::now(),
        });
        repo.insert(&category).await.expect("insert");

        let use_case = GetCategoryUseCase::new(repo);
        let output = use_case
            .execute(GetCategoryInput {
                id: category.category_id().to_string(),
            })
            .await
            .expect("should find");

        assert_eq!(output.id, category.category_id().to_string());
        assert_eq!(output.name, "Movie");
    }

    #[tokio::test]
    async fn should_error_when_not_found() {
        let repo = CategoryInMemoryRepository::new();
        let use_case = GetCategoryUseCase::new(repo);

        let result = use_case
            .execute(GetCategoryInput {
                id: CategoryId::new().to_string(),
            })
            .await;

        assert!(result.is_err());
    }
}
