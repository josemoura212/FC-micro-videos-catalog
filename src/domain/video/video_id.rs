use crate::domain::shared::value_object::{InvalidUuidError, UuidVo};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VideoId(UuidVo);

impl VideoId {
    #[must_use]
    pub fn new() -> Self {
        Self(UuidVo::new())
    }

    /// # Errors
    /// Returns `InvalidUuidError` if the string is not a valid UUID.
    pub fn from(id: &str) -> Result<Self, InvalidUuidError> {
        Ok(Self(UuidVo::from(id)?))
    }

    #[must_use]
    pub const fn inner(&self) -> &UuidVo {
        &self.0
    }
}

impl Default for VideoId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for VideoId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_new_video_id() {
        let id = VideoId::new();
        assert!(!id.to_string().is_empty());
    }

    #[test]
    fn should_create_from_valid_string() {
        let id_str = "4e9e2e4e-0d1a-4a4b-8c0a-5b0e4e4e4e4e";
        let id = VideoId::from(id_str).unwrap();
        assert_eq!(id.to_string(), id_str);
    }

    #[test]
    fn should_compare_equal_ids() {
        let id_str = "4e9e2e4e-0d1a-4a4b-8c0a-5b0e4e4e4e4e";
        let id1 = VideoId::from(id_str).unwrap();
        let id2 = VideoId::from(id_str).unwrap();
        assert_eq!(id1, id2);
    }
}
