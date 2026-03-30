use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::cast_member::aggregate::CastMember;
use crate::domain::cast_member::cast_member_id::CastMemberId;
use crate::domain::cast_member::cast_member_type::CastMemberType;
use crate::domain::shared::errors::LoadEntityError;

pub const CAST_MEMBER_DOCUMENT_TYPE: &str = "CastMember";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastMemberDocument {
    pub cast_member_name: String,
    pub cast_member_type: u8,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(rename = "type")]
    pub doc_type: String,
}

pub struct CastMemberElasticSearchMapper;

impl CastMemberElasticSearchMapper {
    /// # Errors
    /// Returns `LoadEntityError` if the document type is invalid or entity fails validation.
    pub fn to_entity(id: &str, document: &CastMemberDocument) -> Result<CastMember, LoadEntityError> {
        if document.doc_type != CAST_MEMBER_DOCUMENT_TYPE {
            return Err(LoadEntityError {
                errors: vec!["Invalid document type".to_string()],
            });
        }

        let cast_member_id = CastMemberId::from(id).map_err(|e| LoadEntityError {
            errors: vec![e.to_string()],
        })?;

        let cast_member_type =
            CastMemberType::from_u8(document.cast_member_type).map_err(|e| LoadEntityError {
                errors: vec![e.to_string()],
            })?;

        let cast_member = CastMember::new(
            cast_member_id,
            document.cast_member_name.clone(),
            cast_member_type,
            document.created_at,
            document.deleted_at,
        );

        Ok(cast_member)
    }

    #[must_use]
    pub fn to_document(entity: &CastMember) -> CastMemberDocument {
        CastMemberDocument {
            cast_member_name: entity.name().to_string(),
            cast_member_type: entity.cast_member_type() as u8,
            created_at: entity.created_at(),
            deleted_at: entity.deleted_at(),
            doc_type: CAST_MEMBER_DOCUMENT_TYPE.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::cast_member::aggregate::CastMemberCreateCommand;

    #[test]
    fn should_convert_entity_to_document() {
        let cast_member = CastMember::create(CastMemberCreateCommand {
            cast_member_id: CastMemberId::new(),
            name: "John Doe".to_string(),
            cast_member_type: CastMemberType::Actor,
            created_at: Utc::now(),
        });

        let doc = CastMemberElasticSearchMapper::to_document(&cast_member);
        assert_eq!(doc.cast_member_name, "John Doe");
        assert_eq!(doc.cast_member_type, 2);
        assert!(doc.deleted_at.is_none());
        assert_eq!(doc.doc_type, "CastMember");
    }

    #[test]
    fn should_convert_document_to_entity() {
        let id = CastMemberId::new();
        let doc = CastMemberDocument {
            cast_member_name: "John Doe".to_string(),
            cast_member_type: 2,
            created_at: Utc::now(),
            deleted_at: None,
            doc_type: CAST_MEMBER_DOCUMENT_TYPE.to_string(),
        };

        let entity =
            CastMemberElasticSearchMapper::to_entity(&id.to_string(), &doc).expect("should map");
        assert_eq!(entity.name(), "John Doe");
        assert_eq!(entity.cast_member_type(), CastMemberType::Actor);
    }

    #[test]
    fn should_fail_with_invalid_type() {
        let doc = CastMemberDocument {
            cast_member_name: "John Doe".to_string(),
            cast_member_type: 2,
            created_at: Utc::now(),
            deleted_at: None,
            doc_type: "Invalid".to_string(),
        };

        let result = CastMemberElasticSearchMapper::to_entity("some-id", &doc);
        assert!(result.is_err());
    }
}
