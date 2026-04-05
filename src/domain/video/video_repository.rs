use async_trait::async_trait;

use super::aggregate::Video;
use super::video_id::VideoId;
use crate::domain::shared::repository::{
    ExistsByIdResult, FindByIdsResult, SortOrder,
};

#[async_trait]
pub trait IVideoRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;

    fn sortable_fields(&self) -> &[&str];
    async fn insert(&self, entity: &Video) -> Result<(), Self::Error>;
    async fn bulk_insert(&self, entities: &[Video]) -> Result<(), Self::Error>;
    async fn find_by_id(&self, id: &VideoId) -> Result<Option<Video>, Self::Error>;
    async fn find_one_by(
        &self,
        video_id: Option<&VideoId>,
        is_published: Option<bool>,
    ) -> Result<Option<Video>, Self::Error>;
    async fn find_by(
        &self,
        video_id: Option<&VideoId>,
        is_published: Option<bool>,
        order: Option<&SortOrder>,
    ) -> Result<Vec<Video>, Self::Error>;
    async fn find_all(&self) -> Result<Vec<Video>, Self::Error>;
    async fn find_by_ids(
        &self,
        ids: &[VideoId],
    ) -> Result<FindByIdsResult<Video>, Self::Error>;
    async fn exists_by_id(
        &self,
        ids: &[VideoId],
    ) -> Result<ExistsByIdResult, Self::Error>;
    async fn update(&self, entity: &Video) -> Result<(), Self::Error>;
    async fn delete(&self, id: &VideoId) -> Result<(), Self::Error>;
}
