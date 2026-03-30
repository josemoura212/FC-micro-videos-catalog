use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CastMemberType {
    Director = 1,
    Actor = 2,
}

impl CastMemberType {
    /// # Errors
    /// Returns `InvalidCastMemberTypeError` if the value is not 1 or 2.
    pub const fn from_u8(value: u8) -> Result<Self, InvalidCastMemberTypeError> {
        match value {
            1 => Ok(Self::Director),
            2 => Ok(Self::Actor),
            _ => Err(InvalidCastMemberTypeError(value)),
        }
    }
}

impl fmt::Display for CastMemberType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Director => write!(f, "Director"),
            Self::Actor => write!(f, "Actor"),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Invalid cast member type: {0}. Valid values are 1 (Director) or 2 (Actor)")]
pub struct InvalidCastMemberTypeError(pub u8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_director_from_u8() {
        let cast_member_type = CastMemberType::from_u8(1).unwrap();
        assert_eq!(cast_member_type, CastMemberType::Director);
    }

    #[test]
    fn should_create_actor_from_u8() {
        let cast_member_type = CastMemberType::from_u8(2).unwrap();
        assert_eq!(cast_member_type, CastMemberType::Actor);
    }

    #[test]
    fn should_fail_with_invalid_value() {
        let result = CastMemberType::from_u8(0);
        assert!(result.is_err());

        let result = CastMemberType::from_u8(3);
        assert!(result.is_err());
    }

    #[test]
    fn should_display_director() {
        assert_eq!(CastMemberType::Director.to_string(), "Director");
    }

    #[test]
    fn should_display_actor() {
        assert_eq!(CastMemberType::Actor.to_string(), "Actor");
    }
}
