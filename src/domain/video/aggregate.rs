use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::domain::cast_member::cast_member_id::CastMemberId;
use crate::domain::cast_member::nested_cast_member::{NestedCastMember, NestedCastMemberCreateCommand};
use crate::domain::category::category_id::CategoryId;
use crate::domain::category::nested_category::{NestedCategory, NestedCategoryCreateCommand};
use crate::domain::genre::genre_id::GenreId;
use crate::domain::genre::nested_genre::{NestedGenre, NestedGenreCreateCommand};
use crate::domain::shared::entity::{AggregateRoot, Entity};
use crate::domain::shared::notification::Notification;
use crate::domain::shared::value_object::UuidVo;

use super::rating::Rating;
use super::video_id::VideoId;

#[derive(Debug, Clone)]
#[allow(clippy::struct_field_names)]
pub struct Video {
    video_id: VideoId,
    title: String,
    description: String,
    year_launched: i32,
    duration: i32,
    rating: Rating,
    is_opened: bool,
    is_published: bool,
    banner_url: Option<String>,
    thumbnail_url: Option<String>,
    thumbnail_half_url: Option<String>,
    trailer_url: String,
    video_url: String,
    categories: HashMap<String, NestedCategory>,
    genres: HashMap<String, NestedGenre>,
    cast_members: HashMap<String, NestedCastMember>,
    created_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
    notification: Notification,
}

#[derive(Debug, Clone)]
pub struct VideoCreateCommand {
    pub video_id: VideoId,
    pub title: String,
    pub description: String,
    pub year_launched: i32,
    pub duration: i32,
    pub rating: Rating,
    pub is_opened: bool,
    pub is_published: bool,
    pub banner_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub thumbnail_half_url: Option<String>,
    pub trailer_url: String,
    pub video_url: String,
    pub categories_props: Vec<NestedCategoryCreateCommand>,
    pub genres_props: Vec<NestedGenreCreateCommand>,
    pub cast_members_props: Vec<NestedCastMemberCreateCommand>,
    pub created_at: DateTime<Utc>,
}

impl Video {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        video_id: VideoId,
        title: String,
        description: String,
        year_launched: i32,
        duration: i32,
        rating: Rating,
        is_opened: bool,
        is_published: bool,
        banner_url: Option<String>,
        thumbnail_url: Option<String>,
        thumbnail_half_url: Option<String>,
        trailer_url: String,
        video_url: String,
        categories: HashMap<String, NestedCategory>,
        genres: HashMap<String, NestedGenre>,
        cast_members: HashMap<String, NestedCastMember>,
        created_at: DateTime<Utc>,
        deleted_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            video_id,
            title,
            description,
            year_launched,
            duration,
            rating,
            is_opened,
            is_published,
            banner_url,
            thumbnail_url,
            thumbnail_half_url,
            trailer_url,
            video_url,
            categories,
            genres,
            cast_members,
            created_at,
            deleted_at,
            notification: Notification::new(),
        }
    }

    #[must_use]
    pub fn create(command: VideoCreateCommand) -> Self {
        let categories = command
            .categories_props
            .into_iter()
            .map(|props| {
                let key = props.category_id.to_string();
                let nested = NestedCategory::create(props);
                (key, nested)
            })
            .collect();

        let genres = command
            .genres_props
            .into_iter()
            .map(|props| {
                let key = props.genre_id.to_string();
                let nested = NestedGenre::create(props);
                (key, nested)
            })
            .collect();

        let cast_members = command
            .cast_members_props
            .into_iter()
            .map(|props| {
                let key = props.cast_member_id.to_string();
                let nested = NestedCastMember::create(props);
                (key, nested)
            })
            .collect();

        let mut video = Self::new(
            command.video_id,
            command.title,
            command.description,
            command.year_launched,
            command.duration,
            command.rating,
            command.is_opened,
            command.is_published,
            command.banner_url,
            command.thumbnail_url,
            command.thumbnail_half_url,
            command.trailer_url,
            command.video_url,
            categories,
            genres,
            cast_members,
            command.created_at,
            None,
        );
        video.validate();
        video
    }

    pub fn change_title(&mut self, title: String) {
        self.title = title;
        self.validate();
    }

    pub fn change_description(&mut self, description: String) {
        self.description = description;
    }

    pub const fn change_year_launched(&mut self, year_launched: i32) {
        self.year_launched = year_launched;
    }

    pub const fn change_duration(&mut self, duration: i32) {
        self.duration = duration;
    }

    pub fn change_rating(&mut self, rating: Rating) {
        self.rating = rating;
    }

    pub const fn mark_as_opened(&mut self) {
        self.is_opened = true;
    }

    pub const fn mark_as_not_opened(&mut self) {
        self.is_opened = false;
    }

    pub const fn publish(&mut self) {
        self.is_published = true;
    }

    pub const fn unpublish(&mut self) {
        self.is_published = false;
    }

    pub fn replace_banner_url(&mut self, banner_url: Option<String>) {
        self.banner_url = banner_url;
    }

    pub fn replace_thumbnail_url(&mut self, thumbnail_url: Option<String>) {
        self.thumbnail_url = thumbnail_url;
    }

    pub fn replace_thumbnail_half_url(&mut self, thumbnail_half_url: Option<String>) {
        self.thumbnail_half_url = thumbnail_half_url;
    }

    pub fn replace_trailer_url(&mut self, trailer_url: String) {
        self.trailer_url = trailer_url;
    }

    pub fn replace_video_url(&mut self, video_url: String) {
        self.video_url = video_url;
    }

    pub fn add_nested_category(&mut self, nested: NestedCategory) {
        let key = nested.category_id().to_string();
        self.categories.insert(key, nested);
    }

    pub fn remove_nested_category(&mut self, category_id: &CategoryId) {
        self.categories.remove(&category_id.to_string());
    }

    pub fn sync_nested_categories(&mut self, categories: Vec<NestedCategory>) {
        self.categories = categories
            .into_iter()
            .map(|nested| {
                let key = nested.category_id().to_string();
                (key, nested)
            })
            .collect();
    }

    pub fn add_nested_genre(&mut self, nested: NestedGenre) {
        let key = nested.genre_id().to_string();
        self.genres.insert(key, nested);
    }

    pub fn remove_nested_genre(&mut self, genre_id: &GenreId) {
        self.genres.remove(&genre_id.to_string());
    }

    pub fn sync_nested_genres(&mut self, genres: Vec<NestedGenre>) {
        self.genres = genres
            .into_iter()
            .map(|nested| {
                let key = nested.genre_id().to_string();
                (key, nested)
            })
            .collect();
    }

    pub fn add_nested_cast_member(&mut self, nested: NestedCastMember) {
        let key = nested.cast_member_id().to_string();
        self.cast_members.insert(key, nested);
    }

    pub fn remove_nested_cast_member(&mut self, cast_member_id: &CastMemberId) {
        self.cast_members.remove(&cast_member_id.to_string());
    }

    pub fn sync_nested_cast_members(&mut self, cast_members: Vec<NestedCastMember>) {
        self.cast_members = cast_members
            .into_iter()
            .map(|nested| {
                let key = nested.cast_member_id().to_string();
                (key, nested)
            })
            .collect();
    }

    pub fn mark_as_deleted(&mut self) {
        self.deleted_at = Some(Utc::now());
    }

    pub const fn mark_as_not_deleted(&mut self) {
        self.deleted_at = None;
    }

    pub const fn change_created_at(&mut self, created_at: DateTime<Utc>) {
        self.created_at = created_at;
    }

    pub fn validate(&mut self) {
        if self.title.len() > 255 {
            self.notification
                .add_error("title must be shorter than or equal to 255 characters", Some("title"));
        }
    }

    #[must_use]
    pub const fn video_id(&self) -> &VideoId {
        &self.video_id
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    #[must_use]
    pub const fn year_launched(&self) -> i32 {
        self.year_launched
    }

    #[must_use]
    pub const fn duration(&self) -> i32 {
        self.duration
    }

    #[must_use]
    pub const fn rating(&self) -> &Rating {
        &self.rating
    }

    #[must_use]
    pub const fn is_opened(&self) -> bool {
        self.is_opened
    }

    #[must_use]
    pub const fn is_published(&self) -> bool {
        self.is_published
    }

    #[must_use]
    pub fn banner_url(&self) -> Option<&str> {
        self.banner_url.as_deref()
    }

    #[must_use]
    pub fn thumbnail_url(&self) -> Option<&str> {
        self.thumbnail_url.as_deref()
    }

    #[must_use]
    pub fn thumbnail_half_url(&self) -> Option<&str> {
        self.thumbnail_half_url.as_deref()
    }

    #[must_use]
    pub fn trailer_url(&self) -> &str {
        &self.trailer_url
    }

    #[must_use]
    pub fn video_url(&self) -> &str {
        &self.video_url
    }

    #[must_use]
    pub const fn categories(&self) -> &HashMap<String, NestedCategory> {
        &self.categories
    }

    #[must_use]
    pub const fn genres(&self) -> &HashMap<String, NestedGenre> {
        &self.genres
    }

    #[must_use]
    pub const fn cast_members(&self) -> &HashMap<String, NestedCastMember> {
        &self.cast_members
    }

    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[must_use]
    pub const fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }
}

impl Entity for Video {
    fn entity_id(&self) -> &UuidVo {
        self.video_id.inner()
    }

    fn notification(&self) -> &Notification {
        &self.notification
    }

    fn notification_mut(&mut self) -> &mut Notification {
        &mut self.notification
    }
}

impl AggregateRoot for Video {}

#[cfg(test)]
mod tests {
    use crate::domain::cast_member::cast_member_type::CastMemberType;

    use super::*;

    fn make_category_props() -> NestedCategoryCreateCommand {
        NestedCategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            is_active: true,
            deleted_at: None,
        }
    }

    fn make_genre_props() -> NestedGenreCreateCommand {
        NestedGenreCreateCommand {
            genre_id: GenreId::new(),
            name: "Action".to_string(),
            is_active: true,
            deleted_at: None,
        }
    }

    fn make_cast_member_props() -> NestedCastMemberCreateCommand {
        NestedCastMemberCreateCommand {
            cast_member_id: CastMemberId::new(),
            name: "John Doe".to_string(),
            cast_member_type: CastMemberType::Actor,
            deleted_at: None,
        }
    }

    fn make_command() -> VideoCreateCommand {
        VideoCreateCommand {
            video_id: VideoId::new(),
            title: "My Video".to_string(),
            description: "A great video".to_string(),
            year_launched: 2024,
            duration: 120,
            rating: Rating::create_12(),
            is_opened: false,
            is_published: false,
            banner_url: Some("http://banner.jpg".to_string()),
            thumbnail_url: Some("http://thumb.jpg".to_string()),
            thumbnail_half_url: Some("http://thumb_half.jpg".to_string()),
            trailer_url: "http://trailer.mp4".to_string(),
            video_url: "http://video.mp4".to_string(),
            categories_props: vec![make_category_props()],
            genres_props: vec![make_genre_props()],
            cast_members_props: vec![make_cast_member_props()],
            created_at: Utc::now(),
        }
    }

    #[test]
    fn should_create_video() {
        let command = make_command();
        let video = Video::create(command);
        assert_eq!(video.title(), "My Video");
        assert_eq!(video.description(), "A great video");
        assert_eq!(video.year_launched(), 2024);
        assert_eq!(video.duration(), 120);
        assert_eq!(video.rating(), &Rating::R12);
        assert!(!video.is_opened());
        assert!(!video.is_published());
        assert_eq!(video.banner_url(), Some("http://banner.jpg"));
        assert_eq!(video.thumbnail_url(), Some("http://thumb.jpg"));
        assert_eq!(video.thumbnail_half_url(), Some("http://thumb_half.jpg"));
        assert_eq!(video.trailer_url(), "http://trailer.mp4");
        assert_eq!(video.video_url(), "http://video.mp4");
        assert_eq!(video.categories().len(), 1);
        assert_eq!(video.genres().len(), 1);
        assert_eq!(video.cast_members().len(), 1);
        assert!(video.deleted_at().is_none());
        assert!(!video.notification().has_errors());
    }

    #[test]
    fn should_create_video_without_nested() {
        let command = VideoCreateCommand {
            categories_props: vec![],
            genres_props: vec![],
            cast_members_props: vec![],
            ..make_command()
        };
        let video = Video::create(command);
        assert!(video.categories().is_empty());
        assert!(video.genres().is_empty());
        assert!(video.cast_members().is_empty());
    }

    #[test]
    fn should_fail_with_title_too_long() {
        let command = VideoCreateCommand {
            title: "a".repeat(256),
            ..make_command()
        };
        let video = Video::create(command);
        assert!(video.notification().has_errors());
    }

    #[test]
    fn should_change_title() {
        let mut video = Video::create(make_command());
        video.change_title("New Title".to_string());
        assert_eq!(video.title(), "New Title");
    }

    #[test]
    fn should_change_description() {
        let mut video = Video::create(make_command());
        video.change_description("New description".to_string());
        assert_eq!(video.description(), "New description");
    }

    #[test]
    fn should_change_year_launched() {
        let mut video = Video::create(make_command());
        video.change_year_launched(2025);
        assert_eq!(video.year_launched(), 2025);
    }

    #[test]
    fn should_change_duration() {
        let mut video = Video::create(make_command());
        video.change_duration(90);
        assert_eq!(video.duration(), 90);
    }

    #[test]
    fn should_change_rating() {
        let mut video = Video::create(make_command());
        video.change_rating(Rating::create_18());
        assert_eq!(video.rating(), &Rating::R18);
    }

    #[test]
    fn should_mark_as_opened_and_not_opened() {
        let mut video = Video::create(make_command());
        assert!(!video.is_opened());
        video.mark_as_opened();
        assert!(video.is_opened());
        video.mark_as_not_opened();
        assert!(!video.is_opened());
    }

    #[test]
    fn should_publish_and_unpublish() {
        let mut video = Video::create(make_command());
        assert!(!video.is_published());
        video.publish();
        assert!(video.is_published());
        video.unpublish();
        assert!(!video.is_published());
    }

    #[test]
    fn should_replace_banner_url() {
        let mut video = Video::create(make_command());
        video.replace_banner_url(Some("http://new-banner.jpg".to_string()));
        assert_eq!(video.banner_url(), Some("http://new-banner.jpg"));
        video.replace_banner_url(None);
        assert!(video.banner_url().is_none());
    }

    #[test]
    fn should_replace_thumbnail_url() {
        let mut video = Video::create(make_command());
        video.replace_thumbnail_url(Some("http://new-thumb.jpg".to_string()));
        assert_eq!(video.thumbnail_url(), Some("http://new-thumb.jpg"));
    }

    #[test]
    fn should_replace_thumbnail_half_url() {
        let mut video = Video::create(make_command());
        video.replace_thumbnail_half_url(Some("http://new-thumb-half.jpg".to_string()));
        assert_eq!(video.thumbnail_half_url(), Some("http://new-thumb-half.jpg"));
    }

    #[test]
    fn should_replace_trailer_url() {
        let mut video = Video::create(make_command());
        video.replace_trailer_url("http://new-trailer.mp4".to_string());
        assert_eq!(video.trailer_url(), "http://new-trailer.mp4");
    }

    #[test]
    fn should_replace_video_url() {
        let mut video = Video::create(make_command());
        video.replace_video_url("http://new-video.mp4".to_string());
        assert_eq!(video.video_url(), "http://new-video.mp4");
    }

    #[test]
    fn should_sync_nested_categories() {
        let mut video = Video::create(make_command());
        assert_eq!(video.categories().len(), 1);

        let new_cat1 = NestedCategory::create(make_category_props());
        let new_cat2 = NestedCategory::create(make_category_props());
        video.sync_nested_categories(vec![new_cat1, new_cat2]);
        assert_eq!(video.categories().len(), 2);
    }

    #[test]
    fn should_sync_nested_genres() {
        let mut video = Video::create(make_command());
        assert_eq!(video.genres().len(), 1);

        let new_genre1 = NestedGenre::create(make_genre_props());
        let new_genre2 = NestedGenre::create(make_genre_props());
        video.sync_nested_genres(vec![new_genre1, new_genre2]);
        assert_eq!(video.genres().len(), 2);
    }

    #[test]
    fn should_sync_nested_cast_members() {
        let mut video = Video::create(make_command());
        assert_eq!(video.cast_members().len(), 1);

        let new_cm1 = NestedCastMember::create(make_cast_member_props());
        let new_cm2 = NestedCastMember::create(make_cast_member_props());
        video.sync_nested_cast_members(vec![new_cm1, new_cm2]);
        assert_eq!(video.cast_members().len(), 2);
    }

    #[test]
    fn should_add_and_remove_nested_category() {
        let mut video = Video::create(VideoCreateCommand {
            categories_props: vec![],
            ..make_command()
        });
        assert!(video.categories().is_empty());

        let props = make_category_props();
        let category_id = props.category_id.clone();
        let nested = NestedCategory::create(props);
        video.add_nested_category(nested);
        assert_eq!(video.categories().len(), 1);

        video.remove_nested_category(&category_id);
        assert!(video.categories().is_empty());
    }

    #[test]
    fn should_add_and_remove_nested_genre() {
        let mut video = Video::create(VideoCreateCommand {
            genres_props: vec![],
            ..make_command()
        });
        assert!(video.genres().is_empty());

        let props = make_genre_props();
        let genre_id = props.genre_id.clone();
        let nested = NestedGenre::create(props);
        video.add_nested_genre(nested);
        assert_eq!(video.genres().len(), 1);

        video.remove_nested_genre(&genre_id);
        assert!(video.genres().is_empty());
    }

    #[test]
    fn should_add_and_remove_nested_cast_member() {
        let mut video = Video::create(VideoCreateCommand {
            cast_members_props: vec![],
            ..make_command()
        });
        assert!(video.cast_members().is_empty());

        let props = make_cast_member_props();
        let cast_member_id = props.cast_member_id.clone();
        let nested = NestedCastMember::create(props);
        video.add_nested_cast_member(nested);
        assert_eq!(video.cast_members().len(), 1);

        video.remove_nested_cast_member(&cast_member_id);
        assert!(video.cast_members().is_empty());
    }

    #[test]
    fn should_mark_as_deleted() {
        let mut video = Video::create(make_command());
        assert!(video.deleted_at().is_none());
        video.mark_as_deleted();
        assert!(video.deleted_at().is_some());
        video.mark_as_not_deleted();
        assert!(video.deleted_at().is_none());
    }

    #[test]
    fn should_change_created_at() {
        let mut video = Video::create(make_command());
        let new_date = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        video.change_created_at(new_date);
        assert_eq!(video.created_at(), new_date);
    }
}
