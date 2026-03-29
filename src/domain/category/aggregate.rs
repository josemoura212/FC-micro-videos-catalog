use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domain::shared::entity::{AggregateRoot, Entity};
use crate::domain::shared::notification::Notification;
use crate::domain::shared::value_object::UuidVo;

use super::category_id::CategoryId;

#[derive(Debug, Clone)]
#[allow(clippy::struct_field_names)]
pub struct Category {
    category_id: CategoryId,
    name: String,
    description: Option<String>,
    is_active: bool,
    created_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
    notification: Notification,
}

#[derive(Debug, Clone)]
pub struct CategoryCreateCommand {
    pub category_id: CategoryId,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryJson {
    pub category_id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Category {
    pub fn new(
        category_id: CategoryId,
        name: String,
        description: Option<String>,
        is_active: bool,
        created_at: DateTime<Utc>,
        deleted_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            category_id,
            name,
            description,
            is_active,
            created_at,
            deleted_at,
            notification: Notification::new(),
        }
    }

    pub fn create(command: CategoryCreateCommand) -> Self {
        let mut category = Self::new(
            command.category_id,
            command.name,
            command.description,
            command.is_active,
            command.created_at,
            None,
        );
        category.validate();
        category
    }

    pub fn change_name(&mut self, name: String) {
        self.name = name;
        self.validate();
    }

    pub fn change_description(&mut self, description: Option<String>) {
        self.description = description;
    }

    pub const fn change_created_at(&mut self, created_at: DateTime<Utc>) {
        self.created_at = created_at;
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

    pub const fn mark_as_not_deleted(&mut self) {
        self.deleted_at = None;
    }

    pub fn validate(&mut self) {
        if self.name.len() > 255 {
            self.notification
                .add_error("name must be shorter than or equal to 255 characters", Some("name"));
        }
    }

    pub const fn category_id(&self) -> &CategoryId {
        &self.category_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub const fn is_active(&self) -> bool {
        self.is_active
    }

    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub const fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    pub fn to_json(&self) -> CategoryJson {
        CategoryJson {
            category_id: self.category_id.to_string(),
            name: self.name.clone(),
            description: self.description.clone(),
            is_active: self.is_active,
            created_at: self.created_at,
            deleted_at: self.deleted_at,
        }
    }
}

impl Entity for Category {
    fn entity_id(&self) -> &UuidVo {
        self.category_id.inner()
    }

    fn notification(&self) -> &Notification {
        &self.notification
    }

    fn notification_mut(&mut self) -> &mut Notification {
        &mut self.notification
    }
}

impl AggregateRoot for Category {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_command() -> CategoryCreateCommand {
        CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            description: Some("some description".to_string()),
            is_active: true,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn should_create_category() {
        let command = make_command();
        let category = Category::create(command);
        assert_eq!(category.name(), "Movie");
        assert_eq!(category.description(), Some("some description"));
        assert!(category.is_active());
        assert!(category.deleted_at().is_none());
        assert!(!category.notification().has_errors());
    }

    #[test]
    fn should_fail_with_name_too_long() {
        let command = CategoryCreateCommand {
            name: "a".repeat(256),
            ..make_command()
        };
        let category = Category::create(command);
        assert!(category.notification().has_errors());
    }

    #[test]
    fn should_change_name() {
        let mut category = Category::create(make_command());
        category.change_name("Documentary".to_string());
        assert_eq!(category.name(), "Documentary");
    }

    #[test]
    fn should_activate_deactivate() {
        let mut category = Category::create(make_command());
        category.deactivate();
        assert!(!category.is_active());
        category.activate();
        assert!(category.is_active());
    }

    #[test]
    fn should_mark_as_deleted() {
        let mut category = Category::create(make_command());
        assert!(category.deleted_at().is_none());
        category.mark_as_deleted();
        assert!(category.deleted_at().is_some());
        category.mark_as_not_deleted();
        assert!(category.deleted_at().is_none());
    }

    #[test]
    fn should_convert_to_json() {
        let category = Category::create(make_command());
        let json = category.to_json();
        assert_eq!(json.name, "Movie");
        assert!(json.deleted_at.is_none());
    }
}
