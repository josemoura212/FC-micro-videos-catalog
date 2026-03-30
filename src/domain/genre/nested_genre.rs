use chrono::{DateTime, Utc};

use crate::domain::shared::entity::Entity;
use crate::domain::shared::notification::Notification;
use crate::domain::shared::value_object::UuidVo;

use super::genre_id::GenreId;

#[derive(Debug, Clone)]
pub struct NestedGenre {
    genre_id: GenreId,
    name: String,
    is_active: bool,
    deleted_at: Option<DateTime<Utc>>,
    notification: Notification,
}

#[derive(Debug, Clone)]
pub struct NestedGenreCreateCommand {
    pub genre_id: GenreId,
    pub name: String,
    pub is_active: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl NestedGenre {
    #[must_use]
    pub fn new(
        genre_id: GenreId,
        name: String,
        is_active: bool,
        deleted_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            genre_id,
            name,
            is_active,
            deleted_at,
            notification: Notification::new(),
        }
    }

    #[must_use]
    pub fn create(command: NestedGenreCreateCommand) -> Self {
        let mut nested = Self::new(
            command.genre_id,
            command.name,
            command.is_active,
            command.deleted_at,
        );
        nested.validate();
        nested
    }

    pub fn change_name(&mut self, name: String) {
        self.name = name;
        self.validate();
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

    #[must_use]
    pub const fn genre_id(&self) -> &GenreId {
        &self.genre_id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.is_active
    }

    #[must_use]
    pub const fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }
}

impl Entity for NestedGenre {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_command() -> NestedGenreCreateCommand {
        NestedGenreCreateCommand {
            genre_id: GenreId::new(),
            name: "Action".to_string(),
            is_active: true,
            deleted_at: None,
        }
    }

    #[test]
    fn should_create_nested_genre() {
        let command = make_command();
        let nested = NestedGenre::create(command);
        assert_eq!(nested.name(), "Action");
        assert!(nested.is_active());
        assert!(nested.deleted_at().is_none());
        assert!(!nested.notification().has_errors());
    }

    #[test]
    fn should_fail_with_name_too_long() {
        let command = NestedGenreCreateCommand {
            name: "a".repeat(256),
            ..make_command()
        };
        let nested = NestedGenre::create(command);
        assert!(nested.notification().has_errors());
    }

    #[test]
    fn should_change_name() {
        let mut nested = NestedGenre::create(make_command());
        nested.change_name("Drama".to_string());
        assert_eq!(nested.name(), "Drama");
    }

    #[test]
    fn should_activate_deactivate() {
        let mut nested = NestedGenre::create(make_command());
        nested.deactivate();
        assert!(!nested.is_active());
        nested.activate();
        assert!(nested.is_active());
    }

    #[test]
    fn should_mark_as_deleted() {
        let mut nested = NestedGenre::create(make_command());
        assert!(nested.deleted_at().is_none());
        nested.mark_as_deleted();
        assert!(nested.deleted_at().is_some());
        nested.mark_as_not_deleted();
        assert!(nested.deleted_at().is_none());
    }
}
