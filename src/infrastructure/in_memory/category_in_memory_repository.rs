use std::sync::Mutex;

use async_trait::async_trait;

use crate::domain::category::aggregate::Category;
use crate::domain::category::category_id::CategoryId;
use crate::domain::category::category_repository::ICategoryRepository;
use crate::domain::shared::errors::NotFoundError;
use crate::domain::shared::repository::{ExistsByIdResult, FindByIdsResult, SortOrder};

#[derive(Debug, thiserror::Error)]
pub enum InMemoryError {
    #[error(transparent)]
    NotFound(#[from] NotFoundError),
    #[error("lock poisoned")]
    LockPoisoned,
}

pub struct CategoryInMemoryRepository {
    items: Mutex<Vec<Category>>,
}

impl CategoryInMemoryRepository {
    #[must_use] 
    pub const fn new() -> Self {
        Self {
            items: Mutex::new(Vec::new()),
        }
    }

    fn lock_items(&self) -> Result<std::sync::MutexGuard<'_, Vec<Category>>, InMemoryError> {
        self.items.lock().map_err(|_| InMemoryError::LockPoisoned)
    }
}

impl Default for CategoryInMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ICategoryRepository for CategoryInMemoryRepository {
    type Error = InMemoryError;

    fn sortable_fields(&self) -> &[&str] {
        &["name", "created_at"]
    }

    async fn insert(&self, entity: &Category) -> Result<(), Self::Error> {
        self.lock_items()?.push(entity.clone());
        Ok(())
    }

    async fn bulk_insert(&self, entities: &[Category]) -> Result<(), Self::Error> {
        self.lock_items()?.extend(entities.iter().cloned());
        Ok(())
    }

    async fn find_by_id(&self, id: &CategoryId) -> Result<Option<Category>, Self::Error> {
        let items = self.lock_items()?;
        let found = items
            .iter()
            .find(|item| item.category_id() == id)
            .cloned();
        drop(items);
        Ok(found)
    }

    async fn find_one_by(
        &self,
        category_id: Option<&CategoryId>,
        is_active: Option<bool>,
    ) -> Result<Option<Category>, Self::Error> {
        let items = self.lock_items()?;
        let found = items
            .iter()
            .find(|item| {
                let id_match = category_id.is_none_or(|id| item.category_id() == id);
                let active_match = is_active.is_none_or(|active| item.is_active() == active);
                id_match && active_match
            })
            .cloned();
        drop(items);
        Ok(found)
    }

    async fn find_by(
        &self,
        category_id: Option<&CategoryId>,
        is_active: Option<bool>,
        order: Option<&SortOrder>,
    ) -> Result<Vec<Category>, Self::Error> {
        let items = self.lock_items()?;
        let mut filtered: Vec<Category> = items
            .iter()
            .filter(|item| {
                let id_match = category_id.is_none_or(|id| item.category_id() == id);
                let active_match = is_active.is_none_or(|active| item.is_active() == active);
                id_match && active_match
            })
            .cloned()
            .collect();
        drop(items);

        if let Some(sort) = order {
            filtered.sort_by(|a, b| {
                let cmp = match sort.field.as_str() {
                    "name" => a.name().cmp(b.name()),
                    "created_at" => a.created_at().cmp(&b.created_at()),
                    _ => std::cmp::Ordering::Equal,
                };
                match sort.direction {
                    crate::domain::shared::repository::SortDirection::Asc => cmp,
                    crate::domain::shared::repository::SortDirection::Desc => cmp.reverse(),
                }
            });
        }

        Ok(filtered)
    }

    async fn find_all(&self) -> Result<Vec<Category>, Self::Error> {
        let items = self.lock_items()?;
        let result = items.clone();
        drop(items);
        Ok(result)
    }

    async fn find_by_ids(
        &self,
        ids: &[CategoryId],
    ) -> Result<FindByIdsResult<Category>, Self::Error> {
        let items = self.lock_items()?;
        let mut exists = Vec::new();
        let mut not_exists = Vec::new();

        for id in ids {
            if let Some(item) = items.iter().find(|item| item.category_id() == id) {
                exists.push(item.clone());
            } else {
                not_exists.push(id.inner().clone());
            }
        }
        drop(items);

        Ok(FindByIdsResult { exists, not_exists })
    }

    async fn exists_by_id(&self, ids: &[CategoryId]) -> Result<ExistsByIdResult, Self::Error> {
        let items = self.lock_items()?;
        let mut exists = Vec::new();
        let mut not_exists = Vec::new();

        for id in ids {
            if items.iter().any(|item| item.category_id() == id) {
                exists.push(id.inner().clone());
            } else {
                not_exists.push(id.inner().clone());
            }
        }
        drop(items);

        Ok(ExistsByIdResult { exists, not_exists })
    }

    async fn update(&self, entity: &Category) -> Result<(), Self::Error> {
        let mut items = self.lock_items()?;
        let pos = items
            .iter()
            .position(|item| item.category_id() == entity.category_id())
            .ok_or_else(|| {
                NotFoundError::new(&entity.category_id().to_string(), "Category")
            })?;
        items[pos] = entity.clone();
        drop(items);
        Ok(())
    }

    async fn delete(&self, id: &CategoryId) -> Result<(), Self::Error> {
        let mut items = self.lock_items()?;
        let pos = items
            .iter()
            .position(|item| item.category_id() == id)
            .ok_or_else(|| NotFoundError::new(&id.to_string(), "Category"))?;
        items.remove(pos);
        drop(items);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::category::aggregate::CategoryCreateCommand;

    fn make_category() -> Category {
        Category::create(CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            description: Some("some description".to_string()),
            is_active: true,
            created_at: Utc::now(),
        })
    }

    #[tokio::test]
    async fn should_insert_and_find_by_id() {
        let repo = CategoryInMemoryRepository::new();
        let category = make_category();
        repo.insert(&category).await.unwrap();

        let found = repo.find_by_id(category.category_id()).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "Movie");
    }

    #[tokio::test]
    async fn should_return_none_when_not_found() {
        let repo = CategoryInMemoryRepository::new();
        let found = repo.find_by_id(&CategoryId::new()).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn should_update_category() {
        let repo = CategoryInMemoryRepository::new();
        let mut category = make_category();
        repo.insert(&category).await.unwrap();

        category.change_name("Documentary".to_string());
        repo.update(&category).await.unwrap();

        let found = repo.find_by_id(category.category_id()).await.unwrap().unwrap();
        assert_eq!(found.name(), "Documentary");
    }

    #[tokio::test]
    async fn should_delete_category() {
        let repo = CategoryInMemoryRepository::new();
        let category = make_category();
        repo.insert(&category).await.unwrap();

        repo.delete(category.category_id()).await.unwrap();
        let found = repo.find_by_id(category.category_id()).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn should_find_by_filter() {
        let repo = CategoryInMemoryRepository::new();
        let category = make_category();
        repo.insert(&category).await.unwrap();

        let found = repo
            .find_one_by(Some(category.category_id()), Some(true))
            .await
            .unwrap();
        assert!(found.is_some());

        let not_found = repo
            .find_one_by(Some(category.category_id()), Some(false))
            .await
            .unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn should_error_on_update_not_found() {
        let repo = CategoryInMemoryRepository::new();
        let category = make_category();
        let result = repo.update(&category).await;
        assert!(result.is_err());
    }
}
