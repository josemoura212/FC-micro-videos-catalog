use async_trait::async_trait;

use super::aggregate::Genre;
use super::genre_id::GenreId;
use crate::domain::shared::repository::{
    ExistsByIdResult, FindByIdsResult, SortOrder,
};

#[async_trait]
pub trait IGenreRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;

    fn sortable_fields(&self) -> &[&str];
    async fn insert(&self, entity: &Genre) -> Result<(), Self::Error>;
    async fn bulk_insert(&self, entities: &[Genre]) -> Result<(), Self::Error>;
    async fn find_by_id(&self, id: &GenreId) -> Result<Option<Genre>, Self::Error>;
    async fn find_one_by(
        &self,
        genre_id: Option<&GenreId>,
        is_active: Option<bool>,
    ) -> Result<Option<Genre>, Self::Error>;
    async fn find_by(
        &self,
        genre_id: Option<&GenreId>,
        is_active: Option<bool>,
        order: Option<&SortOrder>,
    ) -> Result<Vec<Genre>, Self::Error>;
    async fn find_all(&self) -> Result<Vec<Genre>, Self::Error>;
    async fn find_by_ids(
        &self,
        ids: &[GenreId],
    ) -> Result<FindByIdsResult<Genre>, Self::Error>;
    async fn exists_by_id(
        &self,
        ids: &[GenreId],
    ) -> Result<ExistsByIdResult, Self::Error>;
    async fn update(&self, entity: &Genre) -> Result<(), Self::Error>;
    async fn delete(&self, id: &GenreId) -> Result<(), Self::Error>;
}
