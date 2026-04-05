use crate::domain::shared::repository::{SortDirection, SortOrder};
use crate::domain::video::video_repository::IVideoRepository;

use super::video_output::{VideoOutput, VideoOutputMapper};

#[derive(Debug, thiserror::Error)]
pub enum ListVideosError<E: std::error::Error> {
    #[error(transparent)]
    Repository(E),
}

pub struct ListAllVideosUseCase<R: IVideoRepository> {
    repo: R,
}

impl<R: IVideoRepository> ListAllVideosUseCase<R> {
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// # Errors
    /// Returns repository error on failure.
    pub async fn execute(&self) -> Result<Vec<VideoOutput>, ListVideosError<R::Error>> {
        let order = SortOrder {
            field: "title".to_string(),
            direction: SortDirection::Asc,
        };

        let videos = self
            .repo
            .find_by(None, Some(true), Some(&order))
            .await
            .map_err(ListVideosError::Repository)?;

        Ok(videos.iter().map(VideoOutputMapper::to_output).collect())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::video::aggregate::{Video, VideoCreateCommand};
    use crate::domain::video::rating::Rating;
    use crate::domain::video::video_id::VideoId;
    use crate::infrastructure::in_memory::video_in_memory_repository::VideoInMemoryRepository;

    #[tokio::test]
    async fn should_list_all_published_videos() {
        let repo = VideoInMemoryRepository::new();
        let video1 = Video::create(VideoCreateCommand {
            video_id: VideoId::new(),
            title: "Zebra Movie".to_string(),
            description: "Desc".to_string(),
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
            categories_props: vec![],
            genres_props: vec![],
            cast_members_props: vec![],
            created_at: Utc::now(),
        });
        let video2 = Video::create(VideoCreateCommand {
            video_id: VideoId::new(),
            title: "Alpha Movie".to_string(),
            description: "Desc".to_string(),
            year_launched: 2024,
            duration: 90,
            rating: Rating::RL,
            is_opened: true,
            is_published: true,
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
        repo.insert(&video1).await.expect("insert");
        repo.insert(&video2).await.expect("insert");

        let use_case = ListAllVideosUseCase::new(repo);
        let output = use_case.execute().await.expect("should list");

        assert_eq!(output.len(), 2);
        assert_eq!(output[0].title, "Alpha Movie");
        assert_eq!(output[1].title, "Zebra Movie");
    }

    #[tokio::test]
    async fn should_exclude_unpublished_videos() {
        let repo = VideoInMemoryRepository::new();
        let published = Video::create(VideoCreateCommand {
            video_id: VideoId::new(),
            title: "Published".to_string(),
            description: "Desc".to_string(),
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
            categories_props: vec![],
            genres_props: vec![],
            cast_members_props: vec![],
            created_at: Utc::now(),
        });
        let unpublished = Video::create(VideoCreateCommand {
            video_id: VideoId::new(),
            title: "Unpublished".to_string(),
            description: "Desc".to_string(),
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
        repo.insert(&published).await.expect("insert");
        repo.insert(&unpublished).await.expect("insert");

        let use_case = ListAllVideosUseCase::new(repo);
        let output = use_case.execute().await.expect("should list");

        assert_eq!(output.len(), 1);
        assert_eq!(output[0].title, "Published");
    }

    #[tokio::test]
    async fn should_return_empty_when_no_videos() {
        let repo = VideoInMemoryRepository::new();
        let use_case = ListAllVideosUseCase::new(repo);
        let output = use_case.execute().await.expect("should list");
        assert!(output.is_empty());
    }
}
