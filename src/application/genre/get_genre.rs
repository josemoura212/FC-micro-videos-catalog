use crate::domain::genre::genre_id::GenreId;
use crate::domain::genre::genre_repository::IGenreRepository;
use crate::domain::shared::errors::NotFoundError;

use super::genre_output::{GenreOutput, GenreOutputMapper};

#[derive(Debug, Clone)]
pub struct GetGenreInput {
    pub id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GetGenreError<E: std::error::Error> {
    #[error(transparent)]
    NotFound(#[from] NotFoundError),
    #[error("invalid genre id: {0}")]
    InvalidId(String),
    #[error(transparent)]
    Repository(E),
}

pub struct GetGenreUseCase<R: IGenreRepository> {
    repo: R,
}

impl<R: IGenreRepository> GetGenreUseCase<R> {
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// # Errors
    /// Returns `NotFoundError` if genre not found, or repository error.
    pub async fn execute(
        &self,
        input: GetGenreInput,
    ) -> Result<GenreOutput, GetGenreError<R::Error>> {
        let genre_id =
            GenreId::from(&input.id).map_err(|e| GetGenreError::InvalidId(e.to_string()))?;

        let genre = self
            .repo
            .find_one_by(Some(&genre_id), Some(true))
            .await
            .map_err(GetGenreError::Repository)?
            .ok_or_else(|| NotFoundError::new(&input.id, "Genre"))?;

        Ok(GenreOutputMapper::to_output(&genre))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::category::category_id::CategoryId;
    use crate::domain::category::nested_category::NestedCategoryCreateCommand;
    use crate::domain::genre::aggregate::{Genre, GenreCreateCommand};
    use crate::infrastructure::in_memory::genre_in_memory_repository::GenreInMemoryRepository;

    #[tokio::test]
    async fn should_return_genre() {
        let repo = GenreInMemoryRepository::new();
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
        repo.insert(&genre).await.expect("insert");

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
    async fn should_error_when_not_found() {
        let repo = GenreInMemoryRepository::new();
        let use_case = GetGenreUseCase::new(repo);

        let result = use_case
            .execute(GetGenreInput {
                id: GenreId::new().to_string(),
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_error_when_inactive() {
        let repo = GenreInMemoryRepository::new();
        let mut genre = Genre::create(GenreCreateCommand {
            genre_id: GenreId::new(),
            name: "Action".to_string(),
            categories_props: vec![],
            is_active: false,
            created_at: Utc::now(),
        });
        genre.deactivate();
        repo.insert(&genre).await.expect("insert");

        let use_case = GetGenreUseCase::new(repo);
        let result = use_case
            .execute(GetGenreInput {
                id: genre.genre_id().to_string(),
            })
            .await;

        assert!(result.is_err());
    }
}
