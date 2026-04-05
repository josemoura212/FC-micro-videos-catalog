use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rating {
    RL,
    R10,
    R12,
    R14,
    R16,
    R18,
}

impl Rating {
    /// # Errors
    /// Returns `InvalidRatingError` if the string is not a valid rating.
    pub fn from_str(value: &str) -> Result<Self, InvalidRatingError> {
        match value {
            "L" => Ok(Self::RL),
            "10" => Ok(Self::R10),
            "12" => Ok(Self::R12),
            "14" => Ok(Self::R14),
            "16" => Ok(Self::R16),
            "18" => Ok(Self::R18),
            _ => Err(InvalidRatingError(value.to_string())),
        }
    }

    #[must_use]
    pub fn create_rl() -> Self {
        Self::RL
    }

    #[must_use]
    pub fn create_10() -> Self {
        Self::R10
    }

    #[must_use]
    pub fn create_12() -> Self {
        Self::R12
    }

    #[must_use]
    pub fn create_14() -> Self {
        Self::R14
    }

    #[must_use]
    pub fn create_16() -> Self {
        Self::R16
    }

    #[must_use]
    pub fn create_18() -> Self {
        Self::R18
    }

    #[must_use]
    pub const fn value(&self) -> &str {
        match self {
            Self::RL => "L",
            Self::R10 => "10",
            Self::R12 => "12",
            Self::R14 => "14",
            Self::R16 => "16",
            Self::R18 => "18",
        }
    }
}

impl fmt::Display for Rating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Invalid rating: {0}. Valid values are L, 10, 12, 14, 16, 18")]
pub struct InvalidRatingError(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_rl_from_str() {
        let rating = Rating::from_str("L").unwrap();
        assert_eq!(rating, Rating::RL);
        assert_eq!(rating.value(), "L");
    }

    #[test]
    fn should_create_r10_from_str() {
        let rating = Rating::from_str("10").unwrap();
        assert_eq!(rating, Rating::R10);
        assert_eq!(rating.value(), "10");
    }

    #[test]
    fn should_create_r12_from_str() {
        let rating = Rating::from_str("12").unwrap();
        assert_eq!(rating, Rating::R12);
        assert_eq!(rating.value(), "12");
    }

    #[test]
    fn should_create_r14_from_str() {
        let rating = Rating::from_str("14").unwrap();
        assert_eq!(rating, Rating::R14);
        assert_eq!(rating.value(), "14");
    }

    #[test]
    fn should_create_r16_from_str() {
        let rating = Rating::from_str("16").unwrap();
        assert_eq!(rating, Rating::R16);
        assert_eq!(rating.value(), "16");
    }

    #[test]
    fn should_create_r18_from_str() {
        let rating = Rating::from_str("18").unwrap();
        assert_eq!(rating, Rating::R18);
        assert_eq!(rating.value(), "18");
    }

    #[test]
    fn should_fail_with_invalid_rating() {
        let result = Rating::from_str("PG");
        assert!(result.is_err());
    }

    #[test]
    fn should_display_rating() {
        assert_eq!(Rating::RL.to_string(), "L");
        assert_eq!(Rating::R10.to_string(), "10");
        assert_eq!(Rating::R12.to_string(), "12");
        assert_eq!(Rating::R14.to_string(), "14");
        assert_eq!(Rating::R16.to_string(), "16");
        assert_eq!(Rating::R18.to_string(), "18");
    }

    #[test]
    fn should_create_with_factory_methods() {
        assert_eq!(Rating::create_rl(), Rating::RL);
        assert_eq!(Rating::create_10(), Rating::R10);
        assert_eq!(Rating::create_12(), Rating::R12);
        assert_eq!(Rating::create_14(), Rating::R14);
        assert_eq!(Rating::create_16(), Rating::R16);
        assert_eq!(Rating::create_18(), Rating::R18);
    }
}
