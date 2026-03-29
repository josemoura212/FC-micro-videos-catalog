use crate::domain::category::category_id::CategoryId;
use crate::domain::category::category_repository::ICategoryRepository;
use crate::domain::shared::errors::NotFoundError;

#[derive(Debug, Clone)]
pub struct DeleteCategoryInput {
    pub id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteCategoryError<E: std::error::Error> {
    #[error(transparent)]
    NotFound(#[from] NotFoundError),
    #[error("invalid category id: {0}")]
    InvalidId(String),
    #[error(transparent)]
    Repository(E),
}

pub struct DeleteCategoryUseCase<R: ICategoryRepository> {
    repo: R,
}

impl<R: ICategoryRepository> DeleteCategoryUseCase<R> {
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// # Errors
    /// Returns `NotFoundError` if category not found, or repository error.
    pub async fn execute(
        &self,
        input: DeleteCategoryInput,
    ) -> Result<(), DeleteCategoryError<R::Error>> {
        let category_id = CategoryId::from(&input.id)
            .map_err(|e| DeleteCategoryError::InvalidId(e.to_string()))?;

        let mut category = self
            .repo
            .find_by_id(&category_id)
            .await
            .map_err(DeleteCategoryError::Repository)?
            .ok_or_else(|| NotFoundError::new(&input.id, "Category"))?;

        category.mark_as_deleted();

        self.repo
            .update(&category)
            .await
            .map_err(DeleteCategoryError::Repository)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::category::aggregate::{Category, CategoryCreateCommand};
    use crate::infrastructure::in_memory::category_in_memory_repository::CategoryInMemoryRepository;

    #[tokio::test]
    async fn should_delete_category() {
        let repo = CategoryInMemoryRepository::new();
        let category = Category::create(CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            description: None,
            is_active: true,
            created_at: Utc::now(),
        });
        repo.insert(&category).await.expect("insert");

        let use_case = DeleteCategoryUseCase::new(repo);
        use_case
            .execute(DeleteCategoryInput {
                id: category.category_id().to_string(),
            })
            .await
            .expect("should delete");
    }

    #[tokio::test]
    async fn should_error_when_not_found() {
        let repo = CategoryInMemoryRepository::new();
        let use_case = DeleteCategoryUseCase::new(repo);

        let result = use_case
            .execute(DeleteCategoryInput {
                id: CategoryId::new().to_string(),
            })
            .await;

        assert!(result.is_err());
    }
}
