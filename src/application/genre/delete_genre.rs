use crate::domain::genre::genre_id::GenreId;
use crate::domain::genre::genre_repository::IGenreRepository;
use crate::domain::shared::errors::NotFoundError;

#[derive(Debug, Clone)]
pub struct DeleteGenreInput {
    pub id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteGenreError<E: std::error::Error> {
    #[error(transparent)]
    NotFound(#[from] NotFoundError),
    #[error("invalid genre id: {0}")]
    InvalidId(String),
    #[error(transparent)]
    Repository(E),
}

pub struct DeleteGenreUseCase<R: IGenreRepository> {
    repo: R,
}

impl<R: IGenreRepository> DeleteGenreUseCase<R> {
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// # Errors
    /// Returns `NotFoundError` if genre not found, or repository error.
    pub async fn execute(
        &self,
        input: DeleteGenreInput,
    ) -> Result<(), DeleteGenreError<R::Error>> {
        let genre_id = GenreId::from(&input.id)
            .map_err(|e| DeleteGenreError::InvalidId(e.to_string()))?;

        let mut genre = self
            .repo
            .find_by_id(&genre_id)
            .await
            .map_err(DeleteGenreError::Repository)?
            .ok_or_else(|| NotFoundError::new(&input.id, "Genre"))?;

        genre.mark_as_deleted();

        self.repo
            .update(&genre)
            .await
            .map_err(DeleteGenreError::Repository)?;

        Ok(())
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
    async fn should_delete_genre() {
        let repo = GenreInMemoryRepository::new();
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
        repo.insert(&genre).await.expect("insert");

        let use_case = DeleteGenreUseCase::new(repo);
        use_case
            .execute(DeleteGenreInput {
                id: genre.genre_id().to_string(),
            })
            .await
            .expect("should delete");
    }

    #[tokio::test]
    async fn should_error_when_not_found() {
        let repo = GenreInMemoryRepository::new();
        let use_case = DeleteGenreUseCase::new(repo);

        let result = use_case
            .execute(DeleteGenreInput {
                id: GenreId::new().to_string(),
            })
            .await;

        assert!(result.is_err());
    }
}
