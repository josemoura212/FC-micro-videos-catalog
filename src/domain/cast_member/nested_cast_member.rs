use chrono::{DateTime, Utc};

use crate::domain::shared::entity::Entity;
use crate::domain::shared::notification::Notification;
use crate::domain::shared::value_object::UuidVo;

use super::cast_member_id::CastMemberId;
use super::cast_member_type::CastMemberType;

#[derive(Debug, Clone)]
pub struct NestedCastMember {
    cast_member_id: CastMemberId,
    name: String,
    cast_member_type: CastMemberType,
    deleted_at: Option<DateTime<Utc>>,
    notification: Notification,
}

#[derive(Debug, Clone)]
pub struct NestedCastMemberCreateCommand {
    pub cast_member_id: CastMemberId,
    pub name: String,
    pub cast_member_type: CastMemberType,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl NestedCastMember {
    #[must_use]
    pub fn new(
        cast_member_id: CastMemberId,
        name: String,
        cast_member_type: CastMemberType,
        deleted_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            cast_member_id,
            name,
            cast_member_type,
            deleted_at,
            notification: Notification::new(),
        }
    }

    #[must_use]
    pub fn create(command: NestedCastMemberCreateCommand) -> Self {
        let mut nested = Self::new(
            command.cast_member_id,
            command.name,
            command.cast_member_type,
            command.deleted_at,
        );
        nested.validate();
        nested
    }

    pub fn change_name(&mut self, name: String) {
        self.name = name;
        self.validate();
    }

    pub const fn change_type(&mut self, cast_member_type: CastMemberType) {
        self.cast_member_type = cast_member_type;
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
    pub const fn cast_member_id(&self) -> &CastMemberId {
        &self.cast_member_id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn cast_member_type(&self) -> CastMemberType {
        self.cast_member_type
    }

    #[must_use]
    pub const fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }
}

impl Entity for NestedCastMember {
    fn entity_id(&self) -> &UuidVo {
        self.cast_member_id.inner()
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

    fn make_command() -> NestedCastMemberCreateCommand {
        NestedCastMemberCreateCommand {
            cast_member_id: CastMemberId::new(),
            name: "John Doe".to_string(),
            cast_member_type: CastMemberType::Actor,
            deleted_at: None,
        }
    }

    #[test]
    fn should_create_nested_cast_member() {
        let command = make_command();
        let nested = NestedCastMember::create(command);
        assert_eq!(nested.name(), "John Doe");
        assert_eq!(nested.cast_member_type(), CastMemberType::Actor);
        assert!(nested.deleted_at().is_none());
        assert!(!nested.notification().has_errors());
    }

    #[test]
    fn should_fail_with_name_too_long() {
        let command = NestedCastMemberCreateCommand {
            name: "a".repeat(256),
            ..make_command()
        };
        let nested = NestedCastMember::create(command);
        assert!(nested.notification().has_errors());
    }

    #[test]
    fn should_change_name() {
        let mut nested = NestedCastMember::create(make_command());
        nested.change_name("Jane Doe".to_string());
        assert_eq!(nested.name(), "Jane Doe");
    }

    #[test]
    fn should_change_type() {
        let mut nested = NestedCastMember::create(make_command());
        nested.change_type(CastMemberType::Director);
        assert_eq!(nested.cast_member_type(), CastMemberType::Director);
    }

    #[test]
    fn should_mark_as_deleted() {
        let mut nested = NestedCastMember::create(make_command());
        assert!(nested.deleted_at().is_none());
        nested.mark_as_deleted();
        assert!(nested.deleted_at().is_some());
        nested.mark_as_not_deleted();
        assert!(nested.deleted_at().is_none());
    }
}
