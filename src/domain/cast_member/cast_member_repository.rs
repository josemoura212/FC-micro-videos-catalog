use async_trait::async_trait;

use super::aggregate::CastMember;
use super::cast_member_id::CastMemberId;
use super::cast_member_type::CastMemberType;
use crate::domain::shared::repository::{
    ExistsByIdResult, FindByIdsResult, SortOrder,
};

#[async_trait]
pub trait ICastMemberRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;

    fn sortable_fields(&self) -> &[&str];
    async fn insert(&self, entity: &CastMember) -> Result<(), Self::Error>;
    async fn bulk_insert(&self, entities: &[CastMember]) -> Result<(), Self::Error>;
    async fn find_by_id(&self, id: &CastMemberId) -> Result<Option<CastMember>, Self::Error>;
    async fn find_one_by(
        &self,
        cast_member_id: Option<&CastMemberId>,
        cast_member_type: Option<CastMemberType>,
    ) -> Result<Option<CastMember>, Self::Error>;
    async fn find_by(
        &self,
        cast_member_id: Option<&CastMemberId>,
        cast_member_type: Option<CastMemberType>,
        order: Option<&SortOrder>,
    ) -> Result<Vec<CastMember>, Self::Error>;
    async fn find_all(&self) -> Result<Vec<CastMember>, Self::Error>;
    async fn find_by_ids(
        &self,
        ids: &[CastMemberId],
    ) -> Result<FindByIdsResult<CastMember>, Self::Error>;
    async fn exists_by_id(
        &self,
        ids: &[CastMemberId],
    ) -> Result<ExistsByIdResult, Self::Error>;
    async fn update(&self, entity: &CastMember) -> Result<(), Self::Error>;
    async fn delete(&self, id: &CastMemberId) -> Result<(), Self::Error>;
}
