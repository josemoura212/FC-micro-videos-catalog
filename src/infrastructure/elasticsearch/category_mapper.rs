use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::category::aggregate::Category;
use crate::domain::category::category_id::CategoryId;
use crate::domain::shared::errors::LoadEntityError;

pub const CATEGORY_DOCUMENT_TYPE: &str = "Category";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryDocument {
    pub category_name: String,
    pub category_description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(rename = "type")]
    pub doc_type: String,
}

pub struct CategoryElasticSearchMapper;

impl CategoryElasticSearchMapper {
    /// # Errors
    /// Returns `LoadEntityError` if the document type is invalid or the entity fails validation.
    pub fn to_entity(id: &str, document: &CategoryDocument) -> Result<Category, LoadEntityError> {
        if document.doc_type != CATEGORY_DOCUMENT_TYPE {
            return Err(LoadEntityError {
                errors: vec!["Invalid document type".to_string()],
            });
        }

        let category_id = CategoryId::from(id).map_err(|e| LoadEntityError {
            errors: vec![e.to_string()],
        })?;

        let category = Category::new(
            category_id,
            document.category_name.clone(),
            document.category_description.clone(),
            document.is_active,
            document.created_at,
            document.deleted_at,
        );

        Ok(category)
    }

    #[must_use]
    pub fn to_document(entity: &Category) -> CategoryDocument {
        CategoryDocument {
            category_name: entity.name().to_string(),
            category_description: entity.description().map(String::from),
            is_active: entity.is_active(),
            created_at: entity.created_at(),
            deleted_at: entity.deleted_at(),
            doc_type: CATEGORY_DOCUMENT_TYPE.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::category::aggregate::CategoryCreateCommand;

    #[test]
    fn should_convert_entity_to_document() {
        let category = Category::create(CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            description: Some("desc".to_string()),
            is_active: true,
            created_at: Utc::now(),
        });

        let doc = CategoryElasticSearchMapper::to_document(&category);
        assert_eq!(doc.category_name, "Movie");
        assert_eq!(doc.category_description, Some("desc".to_string()));
        assert!(doc.is_active);
        assert!(doc.deleted_at.is_none());
        assert_eq!(doc.doc_type, "Category");
    }

    #[test]
    fn should_convert_document_to_entity() {
        let id = CategoryId::new();
        let doc = CategoryDocument {
            category_name: "Movie".to_string(),
            category_description: Some("desc".to_string()),
            is_active: true,
            created_at: Utc::now(),
            deleted_at: None,
            doc_type: CATEGORY_DOCUMENT_TYPE.to_string(),
        };

        let entity =
            CategoryElasticSearchMapper::to_entity(&id.to_string(), &doc).expect("should map");
        assert_eq!(entity.name(), "Movie");
        assert_eq!(entity.description(), Some("desc"));
        assert!(entity.is_active());
    }

    #[test]
    fn should_fail_with_invalid_type() {
        let doc = CategoryDocument {
            category_name: "Movie".to_string(),
            category_description: None,
            is_active: true,
            created_at: Utc::now(),
            deleted_at: None,
            doc_type: "Invalid".to_string(),
        };

        let result = CategoryElasticSearchMapper::to_entity("some-id", &doc);
        assert!(result.is_err());
    }
}
