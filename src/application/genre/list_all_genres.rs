use crate::domain::genre::genre_repository::IGenreRepository;
use crate::domain::shared::repository::{SortDirection, SortOrder};

use super::genre_output::{GenreOutput, GenreOutputMapper};

#[derive(Debug, thiserror::Error)]
pub enum ListGenresError<E: std::error::Error> {
    #[error(transparent)]
    Repository(E),
}

pub struct ListAllGenresUseCase<R: IGenreRepository> {
    repo: R,
}

impl<R: IGenreRepository> ListAllGenresUseCase<R> {
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// # Errors
    /// Returns repository error on failure.
    pub async fn execute(&self) -> Result<Vec<GenreOutput>, ListGenresError<R::Error>> {
        let order = SortOrder {
            field: "name".to_string(),
            direction: SortDirection::Asc,
        };

        let genres = self
            .repo
            .find_by(None, Some(true), Some(&order))
            .await
            .map_err(ListGenresError::Repository)?;

        Ok(genres.iter().map(GenreOutputMapper::to_output).collect())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::category::category_id::CategoryId;
    use crate::domain::category::nested_category::NestedCategoryCreateCommand;
    use crate::domain::genre::aggregate::{Genre, GenreCreateCommand};
    use crate::domain::genre::genre_id::GenreId;
    use crate::infrastructure::in_memory::genre_in_memory_repository::GenreInMemoryRepository;

    #[tokio::test]
    async fn should_list_all_active_genres() {
        let repo = GenreInMemoryRepository::new();
        let genre1 = Genre::create(GenreCreateCommand {
            genre_id: GenreId::new(),
            name: "Drama".to_string(),
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
            name: "Action".to_string(),
            categories_props: vec![],
            is_active: true,
            created_at: Utc::now(),
        });
        repo.insert(&genre1).await.expect("insert");
        repo.insert(&genre2).await.expect("insert");

        let use_case = ListAllGenresUseCase::new(repo);
        let output = use_case.execute().await.expect("should list");

        assert_eq!(output.len(), 2);
        assert_eq!(output[0].name, "Action");
        assert_eq!(output[1].name, "Drama");
    }

    #[tokio::test]
    async fn should_exclude_inactive_genres() {
        let repo = GenreInMemoryRepository::new();
        let active = Genre::create(GenreCreateCommand {
            genre_id: GenreId::new(),
            name: "Action".to_string(),
            categories_props: vec![],
            is_active: true,
            created_at: Utc::now(),
        });
        let inactive = Genre::create(GenreCreateCommand {
            genre_id: GenreId::new(),
            name: "Horror".to_string(),
            categories_props: vec![],
            is_active: false,
            created_at: Utc::now(),
        });
        repo.insert(&active).await.expect("insert");
        repo.insert(&inactive).await.expect("insert");

        let use_case = ListAllGenresUseCase::new(repo);
        let output = use_case.execute().await.expect("should list");

        assert_eq!(output.len(), 1);
        assert_eq!(output[0].name, "Action");
    }

    #[tokio::test]
    async fn should_return_empty_when_no_genres() {
        let repo = GenreInMemoryRepository::new();
        let use_case = ListAllGenresUseCase::new(repo);
        let output = use_case.execute().await.expect("should list");
        assert!(output.is_empty());
    }
}
