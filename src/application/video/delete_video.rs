use crate::domain::shared::errors::NotFoundError;
use crate::domain::video::video_id::VideoId;
use crate::domain::video::video_repository::IVideoRepository;

#[derive(Debug, Clone)]
pub struct DeleteVideoInput {
    pub id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteVideoError<E: std::error::Error> {
    #[error(transparent)]
    NotFound(#[from] NotFoundError),
    #[error("invalid video id: {0}")]
    InvalidId(String),
    #[error(transparent)]
    Repository(E),
}

pub struct DeleteVideoUseCase<R: IVideoRepository> {
    repo: R,
}

impl<R: IVideoRepository> DeleteVideoUseCase<R> {
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// # Errors
    /// Returns `NotFoundError` if video not found, or repository error.
    pub async fn execute(
        &self,
        input: DeleteVideoInput,
    ) -> Result<(), DeleteVideoError<R::Error>> {
        let video_id = VideoId::from(&input.id)
            .map_err(|e| DeleteVideoError::InvalidId(e.to_string()))?;

        let mut video = self
            .repo
            .find_by_id(&video_id)
            .await
            .map_err(DeleteVideoError::Repository)?
            .ok_or_else(|| NotFoundError::new(&input.id, "Video"))?;

        video.mark_as_deleted();

        self.repo
            .update(&video)
            .await
            .map_err(DeleteVideoError::Repository)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::video::aggregate::{Video, VideoCreateCommand};
    use crate::domain::video::rating::Rating;
    use crate::infrastructure::in_memory::video_in_memory_repository::VideoInMemoryRepository;

    #[tokio::test]
    async fn should_delete_video() {
        let repo = VideoInMemoryRepository::new();
        let video = Video::create(VideoCreateCommand {
            video_id: VideoId::new(),
            title: "Test Video".to_string(),
            description: "Desc".to_string(),
            year_launched: 2024,
            duration: 120,
            rating: Rating::R12,
            is_opened: false,
            is_published: false,
            banner_url: None,
            thumbnail_url: None,
            thumbnail_half_url: None,
            trailer_url: "http://trailer.mp4".to_string(),
            video_url: "http://video.mp4".to_string(),
            categories_props: vec![],
            genres_props: vec![],
            cast_members_props: vec![],
            created_at: Utc::now(),
        });
        repo.insert(&video).await.expect("insert");

        let use_case = DeleteVideoUseCase::new(repo);
        use_case
            .execute(DeleteVideoInput {
                id: video.video_id().to_string(),
            })
            .await
            .expect("should delete");
    }

    #[tokio::test]
    async fn should_error_when_not_found() {
        let repo = VideoInMemoryRepository::new();
        let use_case = DeleteVideoUseCase::new(repo);

        let result = use_case
            .execute(DeleteVideoInput {
                id: VideoId::new().to_string(),
            })
            .await;

        assert!(result.is_err());
    }
}
