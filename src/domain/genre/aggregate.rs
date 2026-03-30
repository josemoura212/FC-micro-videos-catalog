use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::domain::category::category_id::CategoryId;
use crate::domain::category::nested_category::{NestedCategory, NestedCategoryCreateCommand};
use crate::domain::shared::entity::{AggregateRoot, Entity};
use crate::domain::shared::notification::Notification;
use crate::domain::shared::value_object::UuidVo;

use super::genre_id::GenreId;

#[derive(Debug, Clone)]
#[allow(clippy::struct_field_names)]
pub struct Genre {
    genre_id: GenreId,
    name: String,
    categories: HashMap<String, NestedCategory>,
    is_active: bool,
    created_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
    notification: Notification,
}

#[derive(Debug, Clone)]
pub struct GenreCreateCommand {
    pub genre_id: GenreId,
    pub name: String,
    pub categories_props: Vec<NestedCategoryCreateCommand>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl Genre {
    #[must_use]
    pub fn new(
        genre_id: GenreId,
        name: String,
        categories: HashMap<String, NestedCategory>,
        is_active: bool,
        created_at: DateTime<Utc>,
        deleted_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            genre_id,
            name,
            categories,
            is_active,
            created_at,
            deleted_at,
            notification: Notification::new(),
        }
    }

    #[must_use]
    pub fn create(command: GenreCreateCommand) -> Self {
        let categories = command
            .categories_props
            .into_iter()
            .map(|props| {
                let key = props.category_id.to_string();
                let nested = NestedCategory::create(props);
                (key, nested)
            })
            .collect();

        let mut genre = Self::new(
            command.genre_id,
            command.name,
            categories,
            command.is_active,
            command.created_at,
            None,
        );
        genre.validate();
        genre
    }

    pub fn change_name(&mut self, name: String) {
        self.name = name;
        self.validate();
    }

    pub fn add_nested_category(&mut self, nested: NestedCategory) {
        let key = nested.category_id().to_string();
        self.categories.insert(key, nested);
    }

    pub fn remove_nested_category(&mut self, category_id: &CategoryId) {
        self.categories.remove(&category_id.to_string());
    }

    pub fn activate_nested_category(&mut self, category_id: &CategoryId) {
        if let Some(nested) = self.categories.get_mut(&category_id.to_string()) {
            nested.activate();
        }
    }

    pub fn deactivate_nested_category(&mut self, category_id: &CategoryId) {
        if let Some(nested) = self.categories.get_mut(&category_id.to_string()) {
            nested.deactivate();
        }
    }

    pub fn change_nested_category_name(&mut self, category_id: &CategoryId, name: String) {
        if let Some(nested) = self.categories.get_mut(&category_id.to_string()) {
            nested.change_name(name);
        }
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

    pub const fn activate(&mut self) {
        self.is_active = true;
    }

    pub const fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn mark_as_deleted(&mut self) {
        self.deleted_at = Some(Utc::now());
    }

    pub const fn mark_as_undeleted(&mut self) {
        self.deleted_at = None;
    }

    pub const fn change_created_at(&mut self, created_at: DateTime<Utc>) {
        self.created_at = created_at;
    }

    pub fn validate(&mut self) {
        if self.name.len() > 255 {
            self.notification
                .add_error("name must be shorter than or equal to 255 characters", Some("name"));
        }
    }

    #[must_use]
    pub const fn genre_id(&self) -> &GenreId {
        &self.genre_id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn categories(&self) -> &HashMap<String, NestedCategory> {
        &self.categories
    }

    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.is_active
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

impl Entity for Genre {
    fn entity_id(&self) -> &UuidVo {
        self.genre_id.inner()
    }

    fn notification(&self) -> &Notification {
        &self.notification
    }

    fn notification_mut(&mut self) -> &mut Notification {
        &mut self.notification
    }
}

impl AggregateRoot for Genre {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_category_props() -> NestedCategoryCreateCommand {
        NestedCategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            is_active: true,
            deleted_at: None,
        }
    }

    fn make_command() -> GenreCreateCommand {
        GenreCreateCommand {
            genre_id: GenreId::new(),
            name: "Action".to_string(),
            categories_props: vec![make_category_props()],
            is_active: true,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn should_create_genre() {
        let command = make_command();
        let genre = Genre::create(command);
        assert_eq!(genre.name(), "Action");
        assert!(genre.is_active());
        assert!(genre.deleted_at().is_none());
        assert_eq!(genre.categories().len(), 1);
        assert!(!genre.notification().has_errors());
    }

    #[test]
    fn should_create_genre_without_categories() {
        let command = GenreCreateCommand {
            categories_props: vec![],
            ..make_command()
        };
        let genre = Genre::create(command);
        assert_eq!(genre.name(), "Action");
        assert!(genre.categories().is_empty());
    }

    #[test]
    fn should_fail_with_name_too_long() {
        let command = GenreCreateCommand {
            name: "a".repeat(256),
            ..make_command()
        };
        let genre = Genre::create(command);
        assert!(genre.notification().has_errors());
    }

    #[test]
    fn should_change_name() {
        let mut genre = Genre::create(make_command());
        genre.change_name("Drama".to_string());
        assert_eq!(genre.name(), "Drama");
    }

    #[test]
    fn should_add_nested_category() {
        let mut genre = Genre::create(GenreCreateCommand {
            categories_props: vec![],
            ..make_command()
        });
        assert!(genre.categories().is_empty());

        let nested = NestedCategory::create(make_category_props());
        genre.add_nested_category(nested);
        assert_eq!(genre.categories().len(), 1);
    }

    #[test]
    fn should_remove_nested_category() {
        let props = make_category_props();
        let category_id = props.category_id.clone();
        let command = GenreCreateCommand {
            categories_props: vec![props],
            ..make_command()
        };
        let mut genre = Genre::create(command);
        assert_eq!(genre.categories().len(), 1);

        genre.remove_nested_category(&category_id);
        assert!(genre.categories().is_empty());
    }

    #[test]
    fn should_activate_nested_category() {
        let props = NestedCategoryCreateCommand {
            is_active: false,
            ..make_category_props()
        };
        let category_id = props.category_id.clone();
        let command = GenreCreateCommand {
            categories_props: vec![props],
            ..make_command()
        };
        let mut genre = Genre::create(command);

        let nested = genre.categories().get(&category_id.to_string()).unwrap();
        assert!(!nested.is_active());

        genre.activate_nested_category(&category_id);
        let nested = genre.categories().get(&category_id.to_string()).unwrap();
        assert!(nested.is_active());
    }

    #[test]
    fn should_deactivate_nested_category() {
        let props = make_category_props();
        let category_id = props.category_id.clone();
        let command = GenreCreateCommand {
            categories_props: vec![props],
            ..make_command()
        };
        let mut genre = Genre::create(command);

        genre.deactivate_nested_category(&category_id);
        let nested = genre.categories().get(&category_id.to_string()).unwrap();
        assert!(!nested.is_active());
    }

    #[test]
    fn should_change_nested_category_name() {
        let props = make_category_props();
        let category_id = props.category_id.clone();
        let command = GenreCreateCommand {
            categories_props: vec![props],
            ..make_command()
        };
        let mut genre = Genre::create(command);

        genre.change_nested_category_name(&category_id, "Documentary".to_string());
        let nested = genre.categories().get(&category_id.to_string()).unwrap();
        assert_eq!(nested.name(), "Documentary");
    }

    #[test]
    fn should_sync_nested_categories() {
        let mut genre = Genre::create(make_command());
        assert_eq!(genre.categories().len(), 1);

        let new_cat1 = NestedCategory::create(make_category_props());
        let new_cat2 = NestedCategory::create(make_category_props());
        genre.sync_nested_categories(vec![new_cat1, new_cat2]);
        assert_eq!(genre.categories().len(), 2);
    }

    #[test]
    fn should_activate_deactivate() {
        let mut genre = Genre::create(make_command());
        genre.deactivate();
        assert!(!genre.is_active());
        genre.activate();
        assert!(genre.is_active());
    }

    #[test]
    fn should_mark_as_deleted() {
        let mut genre = Genre::create(make_command());
        assert!(genre.deleted_at().is_none());
        genre.mark_as_deleted();
        assert!(genre.deleted_at().is_some());
        genre.mark_as_undeleted();
        assert!(genre.deleted_at().is_none());
    }

    #[test]
    fn should_change_created_at() {
        let mut genre = Genre::create(make_command());
        let new_date = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        genre.change_created_at(new_date);
        assert_eq!(genre.created_at(), new_date);
    }
}
