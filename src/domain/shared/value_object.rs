use std::fmt;

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UuidVo {
    id: Uuid,
}

impl UuidVo {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
        }
    }

    pub fn from(id: &str) -> Result<Self, InvalidUuidError> {
        let parsed =
            Uuid::parse_str(id).map_err(|_| InvalidUuidError(id.to_string()))?;
        Ok(Self { id: parsed })
    }

    pub const fn id(&self) -> &Uuid {
        &self.id
    }
}

impl Default for UuidVo {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for UuidVo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl From<Uuid> for UuidVo {
    fn from(id: Uuid) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Invalid UUID: {0}")]
pub struct InvalidUuidError(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_new_uuid() {
        let uuid = UuidVo::new();
        assert!(!uuid.to_string().is_empty());
    }

    #[test]
    fn should_create_uuid_from_valid_string() {
        let id = "4e9e2e4e-0d1a-4a4b-8c0a-5b0e4e4e4e4e";
        let uuid = UuidVo::from(id).unwrap();
        assert_eq!(uuid.to_string(), id);
    }

    #[test]
    fn should_fail_with_invalid_uuid_string() {
        let result = UuidVo::from("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn should_compare_equal_uuids() {
        let id = "4e9e2e4e-0d1a-4a4b-8c0a-5b0e4e4e4e4e";
        let uuid1 = UuidVo::from(id).unwrap();
        let uuid2 = UuidVo::from(id).unwrap();
        assert_eq!(uuid1, uuid2);
    }

    #[test]
    fn should_compare_different_uuids() {
        let uuid1 = UuidVo::new();
        let uuid2 = UuidVo::new();
        assert_ne!(uuid1, uuid2);
    }
}
