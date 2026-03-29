use crate::domain::category::category_repository::ICategoryRepository;
use crate::domain::shared::repository::{SortDirection, SortOrder};

use super::category_output::{CategoryOutput, CategoryOutputMapper};

#[derive(Debug, thiserror::Error)]
pub enum ListCategoriesError<E: std::error::Error> {
    #[error(transparent)]
    Repository(E),
}

pub struct ListAllCategoriesUseCase<R: ICategoryRepository> {
    repo: R,
}

impl<R: ICategoryRepository> ListAllCategoriesUseCase<R> {
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// # Errors
    /// Returns repository error on failure.
    pub async fn execute(&self) -> Result<Vec<CategoryOutput>, ListCategoriesError<R::Error>> {
        let order = SortOrder {
            field: "name".to_string(),
            direction: SortDirection::Asc,
        };

        let categories = self
            .repo
            .find_by(None, Some(true), Some(&order))
            .await
            .map_err(ListCategoriesError::Repository)?;

        Ok(categories
            .iter()
            .map(CategoryOutputMapper::to_output)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::category::aggregate::{Category, CategoryCreateCommand};
    use crate::domain::category::category_id::CategoryId;
    use crate::infrastructure::in_memory::category_in_memory_repository::CategoryInMemoryRepository;

    #[tokio::test]
    async fn should_list_all_categories() {
        let repo = CategoryInMemoryRepository::new();
        let cat1 = Category::create(CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            description: None,
            is_active: true,
            created_at: Utc::now(),
        });
        let cat2 = Category::create(CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Documentary".to_string(),
            description: None,
            is_active: true,
            created_at: Utc::now(),
        });
        repo.insert(&cat1).await.expect("insert");
        repo.insert(&cat2).await.expect("insert");

        let use_case = ListAllCategoriesUseCase::new(repo);
        let output = use_case.execute().await.expect("should list");

        assert_eq!(output.len(), 2);
    }
}
