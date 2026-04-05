use crate::domain::shared::errors::NotFoundError;
use crate::domain::video::video_id::VideoId;
use crate::domain::video::video_repository::IVideoRepository;

use super::video_output::{VideoOutput, VideoOutputMapper};

#[derive(Debug, Clone)]
pub struct GetVideoInput {
    pub id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GetVideoError<E: std::error::Error> {
    #[error(transparent)]
    NotFound(#[from] NotFoundError),
    #[error("invalid video id: {0}")]
    InvalidId(String),
    #[error(transparent)]
    Repository(E),
}

pub struct GetVideoUseCase<R: IVideoRepository> {
    repo: R,
}

impl<R: IVideoRepository> GetVideoUseCase<R> {
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// # Errors
    /// Returns `NotFoundError` if video not found, or repository error.
    pub async fn execute(
        &self,
        input: GetVideoInput,
    ) -> Result<VideoOutput, GetVideoError<R::Error>> {
        let video_id =
            VideoId::from(&input.id).map_err(|e| GetVideoError::InvalidId(e.to_string()))?;

        let video = self
            .repo
            .find_one_by(Some(&video_id), Some(true))
            .await
            .map_err(GetVideoError::Repository)?
            .ok_or_else(|| NotFoundError::new(&input.id, "Video"))?;

        Ok(VideoOutputMapper::to_output(&video))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::cast_member::cast_member_id::CastMemberId;
    use crate::domain::cast_member::cast_member_type::CastMemberType;
    use crate::domain::cast_member::nested_cast_member::NestedCastMemberCreateCommand;
    use crate::domain::category::category_id::CategoryId;
    use crate::domain::category::nested_category::NestedCategoryCreateCommand;
    use crate::domain::genre::genre_id::GenreId;
    use crate::domain::genre::nested_genre::NestedGenreCreateCommand;
    use crate::domain::video::aggregate::{Video, VideoCreateCommand};
    use crate::domain::video::rating::Rating;
    use crate::infrastructure::in_memory::video_in_memory_repository::VideoInMemoryRepository;

    #[tokio::test]
    async fn should_return_video() {
        let repo = VideoInMemoryRepository::new();
        let cat_id = CategoryId::new();
        let genre_id = GenreId::new();
        let member_id = CastMemberId::new();

        let video = Video::create(VideoCreateCommand {
            video_id: VideoId::new(),
            title: "Test Video".to_string(),
            description: "A description".to_string(),
            year_launched: 2024,
            duration: 120,
            rating: Rating::R12,
            is_opened: false,
            is_published: true,
            banner_url: None,
            thumbnail_url: None,
            thumbnail_half_url: None,
            trailer_url: "http://trailer.mp4".to_string(),
            video_url: "http://video.mp4".to_string(),
            categories_props: vec![NestedCategoryCreateCommand {
                category_id: cat_id.clone(),
                name: "Movie".to_string(),
                is_active: true,
                deleted_at: None,
            }],
            genres_props: vec![NestedGenreCreateCommand {
                genre_id: genre_id.clone(),
                name: "Action".to_string(),
                is_active: true,
                deleted_at: None,
            }],
            cast_members_props: vec![NestedCastMemberCreateCommand {
                cast_member_id: member_id.clone(),
                name: "John Doe".to_string(),
                cast_member_type: CastMemberType::Actor,
                deleted_at: None,
            }],
            created_at: Utc::now(),
        });
        repo.insert(&video).await.expect("insert");

        let use_case = GetVideoUseCase::new(repo);
        let output = use_case
            .execute(GetVideoInput {
                id: video.video_id().to_string(),
            })
            .await
            .expect("should find");

        assert_eq!(output.id, video.video_id().to_string());
        assert_eq!(output.title, "Test Video");
        assert!(output.is_published);
        assert_eq!(output.categories.len(), 1);
        assert_eq!(output.categories[0].id, cat_id.to_string());
        assert_eq!(output.genres.len(), 1);
        assert_eq!(output.genres[0].id, genre_id.to_string());
        assert_eq!(output.cast_members.len(), 1);
        assert_eq!(output.cast_members[0].id, member_id.to_string());
    }

    #[tokio::test]
    async fn should_error_when_not_found() {
        let repo = VideoInMemoryRepository::new();
        let use_case = GetVideoUseCase::new(repo);

        let result = use_case
            .execute(GetVideoInput {
                id: VideoId::new().to_string(),
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_error_when_not_published() {
        let repo = VideoInMemoryRepository::new();
        let video = Video::create(VideoCreateCommand {
            video_id: VideoId::new(),
            title: "Unpublished".to_string(),
            description: "Not published".to_string(),
            year_launched: 2024,
            duration: 90,
            rating: Rating::RL,
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

        let use_case = GetVideoUseCase::new(repo);
        let result = use_case
            .execute(GetVideoInput {
                id: video.video_id().to_string(),
            })
            .await;

        assert!(result.is_err());
    }
}
