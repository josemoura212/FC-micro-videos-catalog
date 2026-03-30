use async_trait::async_trait;

use super::aggregate::Category;
use super::category_id::CategoryId;
use crate::domain::shared::repository::{
    ExistsByIdResult, FindByIdsResult, SortOrder,
};

#[async_trait]
pub trait ICategoryRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;

    fn sortable_fields(&self) -> &[&str];
    async fn insert(&self, entity: &Category) -> Result<(), Self::Error>;
    async fn bulk_insert(&self, entities: &[Category]) -> Result<(), Self::Error>;
    async fn find_by_id(&self, id: &CategoryId) -> Result<Option<Category>, Self::Error>;
    async fn find_one_by(
        &self,
        category_id: Option<&CategoryId>,
        is_active: Option<bool>,
    ) -> Result<Option<Category>, Self::Error>;
    async fn find_by(
        &self,
        category_id: Option<&CategoryId>,
        is_active: Option<bool>,
        order: Option<&SortOrder>,
    ) -> Result<Vec<Category>, Self::Error>;
    async fn find_all(&self) -> Result<Vec<Category>, Self::Error>;
    async fn find_by_ids(
        &self,
        ids: &[CategoryId],
    ) -> Result<FindByIdsResult<Category>, Self::Error>;
    async fn exists_by_id(
        &self,
        ids: &[CategoryId],
    ) -> Result<ExistsByIdResult, Self::Error>;
    async fn update(&self, entity: &Category) -> Result<(), Self::Error>;
    async fn delete(&self, id: &CategoryId) -> Result<(), Self::Error>;
    async fn has_only_one_activate_in_related(
        &self,
        id: &CategoryId,
    ) -> Result<bool, Self::Error>;
    async fn has_only_one_not_deleted_in_related(
        &self,
        id: &CategoryId,
    ) -> Result<bool, Self::Error>;
}
