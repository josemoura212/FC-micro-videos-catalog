use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domain::shared::entity::{AggregateRoot, Entity};
use crate::domain::shared::notification::Notification;
use crate::domain::shared::value_object::UuidVo;

use super::cast_member_id::CastMemberId;
use super::cast_member_type::CastMemberType;

#[derive(Debug, Clone)]
#[allow(clippy::struct_field_names)]
pub struct CastMember {
    cast_member_id: CastMemberId,
    name: String,
    cast_member_type: CastMemberType,
    created_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
    notification: Notification,
}

#[derive(Debug, Clone)]
pub struct CastMemberCreateCommand {
    pub cast_member_id: CastMemberId,
    pub name: String,
    pub cast_member_type: CastMemberType,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CastMemberJson {
    pub cast_member_id: String,
    pub name: String,
    pub cast_member_type: CastMemberType,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CastMember {
    #[must_use]
    pub fn new(
        cast_member_id: CastMemberId,
        name: String,
        cast_member_type: CastMemberType,
        created_at: DateTime<Utc>,
        deleted_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            cast_member_id,
            name,
            cast_member_type,
            created_at,
            deleted_at,
            notification: Notification::new(),
        }
    }

    #[must_use]
    pub fn create(command: CastMemberCreateCommand) -> Self {
        let mut cast_member = Self::new(
            command.cast_member_id,
            command.name,
            command.cast_member_type,
            command.created_at,
            None,
        );
        cast_member.validate();
        cast_member
    }

    pub fn change_name(&mut self, name: String) {
        self.name = name;
        self.validate();
    }

    pub const fn change_type(&mut self, cast_member_type: CastMemberType) {
        self.cast_member_type = cast_member_type;
    }

    pub const fn change_created_at(&mut self, created_at: DateTime<Utc>) {
        self.created_at = created_at;
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
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[must_use]
    pub const fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    #[must_use]
    pub fn to_json(&self) -> CastMemberJson {
        CastMemberJson {
            cast_member_id: self.cast_member_id.to_string(),
            name: self.name.clone(),
            cast_member_type: self.cast_member_type,
            created_at: self.created_at,
            deleted_at: self.deleted_at,
        }
    }
}

impl Entity for CastMember {
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

impl AggregateRoot for CastMember {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_command() -> CastMemberCreateCommand {
        CastMemberCreateCommand {
            cast_member_id: CastMemberId::new(),
            name: "John Doe".to_string(),
            cast_member_type: CastMemberType::Actor,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn should_create_cast_member() {
        let command = make_command();
        let cast_member = CastMember::create(command);
        assert_eq!(cast_member.name(), "John Doe");
        assert_eq!(cast_member.cast_member_type(), CastMemberType::Actor);
        assert!(cast_member.deleted_at().is_none());
        assert!(!cast_member.notification().has_errors());
    }

    #[test]
    fn should_fail_with_name_too_long() {
        let command = CastMemberCreateCommand {
            name: "a".repeat(256),
            ..make_command()
        };
        let cast_member = CastMember::create(command);
        assert!(cast_member.notification().has_errors());
    }

    #[test]
    fn should_change_name() {
        let mut cast_member = CastMember::create(make_command());
        cast_member.change_name("Jane Doe".to_string());
        assert_eq!(cast_member.name(), "Jane Doe");
    }

    #[test]
    fn should_change_type() {
        let mut cast_member = CastMember::create(make_command());
        cast_member.change_type(CastMemberType::Director);
        assert_eq!(cast_member.cast_member_type(), CastMemberType::Director);
    }

    #[test]
    fn should_mark_as_deleted() {
        let mut cast_member = CastMember::create(make_command());
        assert!(cast_member.deleted_at().is_none());
        cast_member.mark_as_deleted();
        assert!(cast_member.deleted_at().is_some());
        cast_member.mark_as_not_deleted();
        assert!(cast_member.deleted_at().is_none());
    }

    #[test]
    fn should_convert_to_json() {
        let cast_member = CastMember::create(make_command());
        let json = cast_member.to_json();
        assert_eq!(json.name, "John Doe");
        assert_eq!(json.cast_member_type, CastMemberType::Actor);
        assert!(json.deleted_at.is_none());
    }
}
