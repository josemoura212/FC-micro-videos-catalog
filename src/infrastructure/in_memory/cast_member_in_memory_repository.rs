use std::sync::Mutex;

use async_trait::async_trait;

use crate::domain::cast_member::aggregate::CastMember;
use crate::domain::cast_member::cast_member_id::CastMemberId;
use crate::domain::cast_member::cast_member_repository::ICastMemberRepository;
use crate::domain::cast_member::cast_member_type::CastMemberType;
use crate::domain::shared::errors::NotFoundError;
use crate::domain::shared::repository::{ExistsByIdResult, FindByIdsResult, SortDirection, SortOrder};

use super::category_in_memory_repository::InMemoryError;

pub struct CastMemberInMemoryRepository {
    items: Mutex<Vec<CastMember>>,
    soft_delete_scope: bool,
}

impl CastMemberInMemoryRepository {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            items: Mutex::new(Vec::new()),
            soft_delete_scope: false,
        }
    }

    fn lock_items(&self) -> Result<std::sync::MutexGuard<'_, Vec<CastMember>>, InMemoryError> {
        self.items.lock().map_err(|_| InMemoryError::LockPoisoned)
    }

    fn apply_scopes(&self, items: Vec<CastMember>) -> Vec<CastMember> {
        if self.soft_delete_scope {
            items
                .into_iter()
                .filter(|item| item.deleted_at().is_none())
                .collect()
        } else {
            items
        }
    }
}

impl Default for CastMemberInMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::domain::shared::criteria::ScopedRepository for CastMemberInMemoryRepository {
    fn ignore_soft_deleted(&mut self) -> &mut Self {
        self.soft_delete_scope = true;
        self
    }

    fn clear_scopes(&mut self) -> &mut Self {
        self.soft_delete_scope = false;
        self
    }
}

#[async_trait]
impl ICastMemberRepository for CastMemberInMemoryRepository {
    type Error = InMemoryError;

    fn sortable_fields(&self) -> &[&str] {
        &["name", "created_at"]
    }

    async fn insert(&self, entity: &CastMember) -> Result<(), Self::Error> {
        self.lock_items()?.push(entity.clone());
        Ok(())
    }

    async fn bulk_insert(&self, entities: &[CastMember]) -> Result<(), Self::Error> {
        self.lock_items()?.extend(entities.iter().cloned());
        Ok(())
    }

    async fn find_by_id(&self, id: &CastMemberId) -> Result<Option<CastMember>, Self::Error> {
        let items = self.lock_items()?;
        let scoped = self.apply_scopes(items.clone());
        drop(items);
        let found = scoped
            .iter()
            .find(|item| item.cast_member_id() == id)
            .cloned();
        Ok(found)
    }

    async fn find_one_by(
        &self,
        cast_member_id: Option<&CastMemberId>,
        cast_member_type: Option<CastMemberType>,
    ) -> Result<Option<CastMember>, Self::Error> {
        let items = self.lock_items()?;
        let scoped = self.apply_scopes(items.clone());
        drop(items);
        let found = scoped
            .iter()
            .find(|item| {
                let id_match = cast_member_id.is_none_or(|id| item.cast_member_id() == id);
                let type_match =
                    cast_member_type.is_none_or(|t| item.cast_member_type() == t);
                id_match && type_match
            })
            .cloned();
        Ok(found)
    }

    async fn find_by(
        &self,
        cast_member_id: Option<&CastMemberId>,
        cast_member_type: Option<CastMemberType>,
        order: Option<&SortOrder>,
    ) -> Result<Vec<CastMember>, Self::Error> {
        let items = self.lock_items()?;
        let scoped = self.apply_scopes(items.clone());
        drop(items);
        let mut filtered: Vec<CastMember> = scoped
            .iter()
            .filter(|item| {
                let id_match = cast_member_id.is_none_or(|id| item.cast_member_id() == id);
                let type_match =
                    cast_member_type.is_none_or(|t| item.cast_member_type() == t);
                id_match && type_match
            })
            .cloned()
            .collect();

        let sort = order.cloned().unwrap_or(SortOrder {
            field: "created_at".to_string(),
            direction: SortDirection::Desc,
        });

        filtered.sort_by(|a, b| {
            let cmp = match sort.field.as_str() {
                "name" => a.name().cmp(b.name()),
                "created_at" => a.created_at().cmp(&b.created_at()),
                _ => std::cmp::Ordering::Equal,
            };
            match sort.direction {
                SortDirection::Asc => cmp,
                SortDirection::Desc => cmp.reverse(),
            }
        });

        Ok(filtered)
    }

    async fn find_all(&self) -> Result<Vec<CastMember>, Self::Error> {
        let items = self.lock_items()?;
        let result = self.apply_scopes(items.clone());
        drop(items);
        Ok(result)
    }

    async fn find_by_ids(
        &self,
        ids: &[CastMemberId],
    ) -> Result<FindByIdsResult<CastMember>, Self::Error> {
        let items = self.lock_items()?;
        let scoped = self.apply_scopes(items.clone());
        drop(items);
        let mut exists = Vec::new();
        let mut not_exists = Vec::new();

        for id in ids {
            if let Some(item) = scoped.iter().find(|item| item.cast_member_id() == id) {
                exists.push(item.clone());
            } else {
                not_exists.push(id.inner().clone());
            }
        }

        Ok(FindByIdsResult { exists, not_exists })
    }

    async fn exists_by_id(&self, ids: &[CastMemberId]) -> Result<ExistsByIdResult, Self::Error> {
        let items = self.lock_items()?;
        let mut exists = Vec::new();
        let mut not_exists = Vec::new();

        for id in ids {
            if items.iter().any(|item| item.cast_member_id() == id) {
                exists.push(id.inner().clone());
            } else {
                not_exists.push(id.inner().clone());
            }
        }
        drop(items);

        Ok(ExistsByIdResult { exists, not_exists })
    }

    async fn update(&self, entity: &CastMember) -> Result<(), Self::Error> {
        let mut items = self.lock_items()?;
        let pos = items
            .iter()
            .position(|item| item.cast_member_id() == entity.cast_member_id())
            .ok_or_else(|| {
                NotFoundError::new(&entity.cast_member_id().to_string(), "CastMember")
            })?;
        items[pos] = entity.clone();
        drop(items);
        Ok(())
    }

    async fn delete(&self, id: &CastMemberId) -> Result<(), Self::Error> {
        let mut items = self.lock_items()?;
        let pos = items
            .iter()
            .position(|item| item.cast_member_id() == id)
            .ok_or_else(|| NotFoundError::new(&id.to_string(), "CastMember"))?;
        items.remove(pos);
        drop(items);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::cast_member::aggregate::CastMemberCreateCommand;

    fn make_cast_member() -> CastMember {
        CastMember::create(CastMemberCreateCommand {
            cast_member_id: CastMemberId::new(),
            name: "John Doe".to_string(),
            cast_member_type: CastMemberType::Actor,
            created_at: Utc::now(),
        })
    }

    #[tokio::test]
    async fn should_insert_and_find_by_id() {
        let repo = CastMemberInMemoryRepository::new();
        let cast_member = make_cast_member();
        repo.insert(&cast_member).await.unwrap();

        let found = repo.find_by_id(cast_member.cast_member_id()).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "John Doe");
    }

    #[tokio::test]
    async fn should_filter_by_type() {
        let repo = CastMemberInMemoryRepository::new();
        let actor = make_cast_member();
        let director = CastMember::create(CastMemberCreateCommand {
            cast_member_id: CastMemberId::new(),
            name: "Jane Director".to_string(),
            cast_member_type: CastMemberType::Director,
            created_at: Utc::now(),
        });
        repo.insert(&actor).await.unwrap();
        repo.insert(&director).await.unwrap();

        let found = repo
            .find_one_by(None, Some(CastMemberType::Director))
            .await
            .unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "Jane Director");
    }

    #[tokio::test]
    async fn should_update_cast_member() {
        let repo = CastMemberInMemoryRepository::new();
        let mut cast_member = make_cast_member();
        repo.insert(&cast_member).await.unwrap();

        cast_member.change_name("Updated Name".to_string());
        repo.update(&cast_member).await.unwrap();

        let found = repo
            .find_by_id(cast_member.cast_member_id())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found.name(), "Updated Name");
    }

    #[tokio::test]
    async fn should_delete_cast_member() {
        let repo = CastMemberInMemoryRepository::new();
        let cast_member = make_cast_member();
        repo.insert(&cast_member).await.unwrap();

        repo.delete(cast_member.cast_member_id()).await.unwrap();
        let found = repo.find_by_id(cast_member.cast_member_id()).await.unwrap();
        assert!(found.is_none());
    }
}
