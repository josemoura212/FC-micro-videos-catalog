use std::sync::Mutex;

use async_trait::async_trait;

use crate::domain::genre::aggregate::Genre;
use crate::domain::genre::genre_id::GenreId;
use crate::domain::genre::genre_repository::IGenreRepository;
use crate::domain::shared::errors::NotFoundError;
use crate::domain::shared::repository::{ExistsByIdResult, FindByIdsResult, SortDirection, SortOrder};

use super::category_in_memory_repository::InMemoryError;

pub struct GenreInMemoryRepository {
    items: Mutex<Vec<Genre>>,
    soft_delete_scope: bool,
}

impl GenreInMemoryRepository {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            items: Mutex::new(Vec::new()),
            soft_delete_scope: false,
        }
    }

    fn lock_items(&self) -> Result<std::sync::MutexGuard<'_, Vec<Genre>>, InMemoryError> {
        self.items.lock().map_err(|_| InMemoryError::LockPoisoned)
    }

    fn apply_scopes(&self, items: Vec<Genre>) -> Vec<Genre> {
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

impl Default for GenreInMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::domain::shared::criteria::ScopedRepository for GenreInMemoryRepository {
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
impl IGenreRepository for GenreInMemoryRepository {
    type Error = InMemoryError;

    fn sortable_fields(&self) -> &[&str] {
        &["name", "created_at"]
    }

    async fn insert(&self, entity: &Genre) -> Result<(), Self::Error> {
        self.lock_items()?.push(entity.clone());
        Ok(())
    }

    async fn bulk_insert(&self, entities: &[Genre]) -> Result<(), Self::Error> {
        self.lock_items()?.extend(entities.iter().cloned());
        Ok(())
    }

    async fn find_by_id(&self, id: &GenreId) -> Result<Option<Genre>, Self::Error> {
        let items = self.lock_items()?;
        let scoped = self.apply_scopes(items.clone());
        drop(items);
        let found = scoped
            .iter()
            .find(|item| item.genre_id() == id)
            .cloned();
        Ok(found)
    }

    async fn find_one_by(
        &self,
        genre_id: Option<&GenreId>,
        is_active: Option<bool>,
    ) -> Result<Option<Genre>, Self::Error> {
        let items = self.lock_items()?;
        let scoped = self.apply_scopes(items.clone());
        drop(items);
        let found = scoped
            .iter()
            .find(|item| {
                let id_match = genre_id.is_none_or(|id| item.genre_id() == id);
                let active_match = is_active.is_none_or(|active| item.is_active() == active);
                id_match && active_match
            })
            .cloned();
        Ok(found)
    }

    async fn find_by(
        &self,
        genre_id: Option<&GenreId>,
        is_active: Option<bool>,
        order: Option<&SortOrder>,
    ) -> Result<Vec<Genre>, Self::Error> {
        let items = self.lock_items()?;
        let scoped = self.apply_scopes(items.clone());
        drop(items);
        let mut filtered: Vec<Genre> = scoped
            .iter()
            .filter(|item| {
                let id_match = genre_id.is_none_or(|id| item.genre_id() == id);
                let active_match = is_active.is_none_or(|active| item.is_active() == active);
                id_match && active_match
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

    async fn find_all(&self) -> Result<Vec<Genre>, Self::Error> {
        let items = self.lock_items()?;
        let result = self.apply_scopes(items.clone());
        drop(items);
        Ok(result)
    }

    async fn find_by_ids(
        &self,
        ids: &[GenreId],
    ) -> Result<FindByIdsResult<Genre>, Self::Error> {
        let items = self.lock_items()?;
        let scoped = self.apply_scopes(items.clone());
        drop(items);
        let mut exists = Vec::new();
        let mut not_exists = Vec::new();

        for id in ids {
            if let Some(item) = scoped.iter().find(|item| item.genre_id() == id) {
                exists.push(item.clone());
            } else {
                not_exists.push(id.inner().clone());
            }
        }

        Ok(FindByIdsResult { exists, not_exists })
    }

    async fn exists_by_id(&self, ids: &[GenreId]) -> Result<ExistsByIdResult, Self::Error> {
        let items = self.lock_items()?;
        let mut exists = Vec::new();
        let mut not_exists = Vec::new();

        for id in ids {
            if items.iter().any(|item| item.genre_id() == id) {
                exists.push(id.inner().clone());
            } else {
                not_exists.push(id.inner().clone());
            }
        }
        drop(items);

        Ok(ExistsByIdResult { exists, not_exists })
    }

    async fn update(&self, entity: &Genre) -> Result<(), Self::Error> {
        let mut items = self.lock_items()?;
        let pos = items
            .iter()
            .position(|item| item.genre_id() == entity.genre_id())
            .ok_or_else(|| {
                NotFoundError::new(&entity.genre_id().to_string(), "Genre")
            })?;
        items[pos] = entity.clone();
        drop(items);
        Ok(())
    }

    async fn delete(&self, id: &GenreId) -> Result<(), Self::Error> {
        let mut items = self.lock_items()?;
        let pos = items
            .iter()
            .position(|item| item.genre_id() == id)
            .ok_or_else(|| NotFoundError::new(&id.to_string(), "Genre"))?;
        items.remove(pos);
        drop(items);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::category::category_id::CategoryId;
    use crate::domain::category::nested_category::NestedCategoryCreateCommand;
    use crate::domain::genre::aggregate::GenreCreateCommand;

    fn make_genre() -> Genre {
        Genre::create(GenreCreateCommand {
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
        })
    }

    #[tokio::test]
    async fn should_insert_and_find_by_id() {
        let repo = GenreInMemoryRepository::new();
        let genre = make_genre();
        repo.insert(&genre).await.unwrap();

        let found = repo.find_by_id(genre.genre_id()).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "Action");
    }

    #[tokio::test]
    async fn should_return_none_when_not_found() {
        let repo = GenreInMemoryRepository::new();
        let found = repo.find_by_id(&GenreId::new()).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn should_filter_by_is_active() {
        let repo = GenreInMemoryRepository::new();
        let genre = make_genre();
        repo.insert(&genre).await.unwrap();

        let found = repo
            .find_one_by(Some(genre.genre_id()), Some(true))
            .await
            .unwrap();
        assert!(found.is_some());

        let not_found = repo
            .find_one_by(Some(genre.genre_id()), Some(false))
            .await
            .unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn should_update_genre() {
        let repo = GenreInMemoryRepository::new();
        let mut genre = make_genre();
        repo.insert(&genre).await.unwrap();

        genre.change_name("Drama".to_string());
        repo.update(&genre).await.unwrap();

        let found = repo
            .find_by_id(genre.genre_id())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found.name(), "Drama");
    }

    #[tokio::test]
    async fn should_delete_genre() {
        let repo = GenreInMemoryRepository::new();
        let genre = make_genre();
        repo.insert(&genre).await.unwrap();

        repo.delete(genre.genre_id()).await.unwrap();
        let found = repo.find_by_id(genre.genre_id()).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn should_error_on_update_not_found() {
        let repo = GenreInMemoryRepository::new();
        let genre = make_genre();
        let result = repo.update(&genre).await;
        assert!(result.is_err());
    }
}
