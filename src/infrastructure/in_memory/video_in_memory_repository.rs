use std::sync::Mutex;

use async_trait::async_trait;

use crate::domain::shared::errors::NotFoundError;
use crate::domain::shared::repository::{
    ExistsByIdResult, FindByIdsResult, SortDirection, SortOrder,
};
use crate::domain::video::aggregate::Video;
use crate::domain::video::video_id::VideoId;
use crate::domain::video::video_repository::IVideoRepository;

use super::category_in_memory_repository::InMemoryError;

pub struct VideoInMemoryRepository {
    items: Mutex<Vec<Video>>,
    soft_delete_scope: bool,
}

impl VideoInMemoryRepository {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            items: Mutex::new(Vec::new()),
            soft_delete_scope: false,
        }
    }

    fn lock_items(&self) -> Result<std::sync::MutexGuard<'_, Vec<Video>>, InMemoryError> {
        self.items.lock().map_err(|_| InMemoryError::LockPoisoned)
    }

    fn apply_scopes(&self, items: Vec<Video>) -> Vec<Video> {
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

impl Default for VideoInMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::domain::shared::criteria::ScopedRepository for VideoInMemoryRepository {
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
impl IVideoRepository for VideoInMemoryRepository {
    type Error = InMemoryError;

    fn sortable_fields(&self) -> &[&str] {
        &["title", "created_at"]
    }

    async fn insert(&self, entity: &Video) -> Result<(), Self::Error> {
        self.lock_items()?.push(entity.clone());
        Ok(())
    }

    async fn bulk_insert(&self, entities: &[Video]) -> Result<(), Self::Error> {
        self.lock_items()?.extend(entities.iter().cloned());
        Ok(())
    }

    async fn find_by_id(&self, id: &VideoId) -> Result<Option<Video>, Self::Error> {
        let items = self.lock_items()?;
        let scoped = self.apply_scopes(items.clone());
        drop(items);
        let found = scoped
            .iter()
            .find(|item| item.video_id() == id)
            .cloned();
        Ok(found)
    }

    async fn find_one_by(
        &self,
        video_id: Option<&VideoId>,
        is_published: Option<bool>,
    ) -> Result<Option<Video>, Self::Error> {
        let items = self.lock_items()?;
        let scoped = self.apply_scopes(items.clone());
        drop(items);
        let found = scoped
            .iter()
            .find(|item| {
                let id_match = video_id.is_none_or(|id| item.video_id() == id);
                let published_match =
                    is_published.is_none_or(|published| item.is_published() == published);
                id_match && published_match
            })
            .cloned();
        Ok(found)
    }

    async fn find_by(
        &self,
        video_id: Option<&VideoId>,
        is_published: Option<bool>,
        order: Option<&SortOrder>,
    ) -> Result<Vec<Video>, Self::Error> {
        let items = self.lock_items()?;
        let scoped = self.apply_scopes(items.clone());
        drop(items);
        let mut filtered: Vec<Video> = scoped
            .iter()
            .filter(|item| {
                let id_match = video_id.is_none_or(|id| item.video_id() == id);
                let published_match =
                    is_published.is_none_or(|published| item.is_published() == published);
                id_match && published_match
            })
            .cloned()
            .collect();

        let sort = order.cloned().unwrap_or(SortOrder {
            field: "created_at".to_string(),
            direction: SortDirection::Desc,
        });

        filtered.sort_by(|a, b| {
            let cmp = match sort.field.as_str() {
                "title" => a.title().cmp(b.title()),
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

    async fn find_all(&self) -> Result<Vec<Video>, Self::Error> {
        let items = self.lock_items()?;
        let result = self.apply_scopes(items.clone());
        drop(items);
        Ok(result)
    }

    async fn find_by_ids(
        &self,
        ids: &[VideoId],
    ) -> Result<FindByIdsResult<Video>, Self::Error> {
        let items = self.lock_items()?;
        let scoped = self.apply_scopes(items.clone());
        drop(items);
        let mut exists = Vec::new();
        let mut not_exists = Vec::new();

        for id in ids {
            if let Some(item) = scoped.iter().find(|item| item.video_id() == id) {
                exists.push(item.clone());
            } else {
                not_exists.push(id.inner().clone());
            }
        }

        Ok(FindByIdsResult { exists, not_exists })
    }

    async fn exists_by_id(&self, ids: &[VideoId]) -> Result<ExistsByIdResult, Self::Error> {
        let items = self.lock_items()?;
        let mut exists = Vec::new();
        let mut not_exists = Vec::new();

        for id in ids {
            if items.iter().any(|item| item.video_id() == id) {
                exists.push(id.inner().clone());
            } else {
                not_exists.push(id.inner().clone());
            }
        }
        drop(items);

        Ok(ExistsByIdResult { exists, not_exists })
    }

    async fn update(&self, entity: &Video) -> Result<(), Self::Error> {
        let mut items = self.lock_items()?;
        let pos = items
            .iter()
            .position(|item| item.video_id() == entity.video_id())
            .ok_or_else(|| {
                NotFoundError::new(&entity.video_id().to_string(), "Video")
            })?;
        items[pos] = entity.clone();
        drop(items);
        Ok(())
    }

    async fn delete(&self, id: &VideoId) -> Result<(), Self::Error> {
        let mut items = self.lock_items()?;
        let pos = items
            .iter()
            .position(|item| item.video_id() == id)
            .ok_or_else(|| NotFoundError::new(&id.to_string(), "Video"))?;
        items.remove(pos);
        drop(items);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::video::aggregate::VideoCreateCommand;
    use crate::domain::video::rating::Rating;

    fn make_video() -> Video {
        Video::create(VideoCreateCommand {
            video_id: VideoId::new(),
            title: "Test Video".to_string(),
            description: "A test description".to_string(),
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
        })
    }

    #[tokio::test]
    async fn should_insert_and_find_by_id() {
        let repo = VideoInMemoryRepository::new();
        let video = make_video();
        repo.insert(&video).await.unwrap();

        let found = repo.find_by_id(video.video_id()).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title(), "Test Video");
    }

    #[tokio::test]
    async fn should_return_none_when_not_found() {
        let repo = VideoInMemoryRepository::new();
        let found = repo.find_by_id(&VideoId::new()).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn should_filter_by_is_published() {
        let repo = VideoInMemoryRepository::new();
        let video = make_video();
        repo.insert(&video).await.unwrap();

        let found = repo
            .find_one_by(Some(video.video_id()), Some(true))
            .await
            .unwrap();
        assert!(found.is_some());

        let not_found = repo
            .find_one_by(Some(video.video_id()), Some(false))
            .await
            .unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn should_update_video() {
        let repo = VideoInMemoryRepository::new();
        let mut video = make_video();
        repo.insert(&video).await.unwrap();

        video.change_title("Updated Title".to_string());
        repo.update(&video).await.unwrap();

        let found = repo
            .find_by_id(video.video_id())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found.title(), "Updated Title");
    }

    #[tokio::test]
    async fn should_delete_video() {
        let repo = VideoInMemoryRepository::new();
        let video = make_video();
        repo.insert(&video).await.unwrap();

        repo.delete(video.video_id()).await.unwrap();
        let found = repo.find_by_id(video.video_id()).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn should_error_on_update_not_found() {
        let repo = VideoInMemoryRepository::new();
        let video = make_video();
        let result = repo.update(&video).await;
        assert!(result.is_err());
    }
}
