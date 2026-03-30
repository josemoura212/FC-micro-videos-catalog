use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::category::category_id::CategoryId;
use crate::domain::category::nested_category::NestedCategory;
use crate::domain::genre::aggregate::Genre;
use crate::domain::genre::genre_id::GenreId;
use crate::domain::shared::errors::LoadEntityError;

pub const GENRE_DOCUMENT_TYPE: &str = "Genre";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedCategoryDocument {
    pub category_id: String,
    pub category_name: String,
    pub is_active: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreDocument {
    pub genre_name: String,
    pub categories: Vec<NestedCategoryDocument>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(rename = "type")]
    pub doc_type: String,
}

pub struct GenreElasticSearchMapper;

impl GenreElasticSearchMapper {
    /// # Errors
    /// Returns `LoadEntityError` if the document type is invalid or the entity fails validation.
    pub fn to_entity(id: &str, document: &GenreDocument) -> Result<Genre, LoadEntityError> {
        if document.doc_type != GENRE_DOCUMENT_TYPE {
            return Err(LoadEntityError {
                errors: vec!["Invalid document type".to_string()],
            });
        }

        let genre_id = GenreId::from(id).map_err(|e| LoadEntityError {
            errors: vec![e.to_string()],
        })?;

        let mut categories = HashMap::new();
        for cat_doc in &document.categories {
            let category_id = CategoryId::from(&cat_doc.category_id).map_err(|e| {
                LoadEntityError {
                    errors: vec![e.to_string()],
                }
            })?;

            let nested = NestedCategory::new(
                category_id,
                cat_doc.category_name.clone(),
                cat_doc.is_active,
                cat_doc.deleted_at,
            );

            categories.insert(cat_doc.category_id.clone(), nested);
        }

        let genre = Genre::new(
            genre_id,
            document.genre_name.clone(),
            categories,
            document.is_active,
            document.created_at,
            document.deleted_at,
        );

        Ok(genre)
    }

    #[must_use]
    pub fn to_document(entity: &Genre) -> GenreDocument {
        let categories: Vec<NestedCategoryDocument> = entity
            .categories()
            .values()
            .map(|nested| NestedCategoryDocument {
                category_id: nested.category_id().to_string(),
                category_name: nested.name().to_string(),
                is_active: nested.is_active(),
                deleted_at: nested.deleted_at(),
                is_deleted: nested.deleted_at().is_some(),
            })
            .collect();

        GenreDocument {
            genre_name: entity.name().to_string(),
            categories,
            is_active: entity.is_active(),
            created_at: entity.created_at(),
            deleted_at: entity.deleted_at(),
            doc_type: GENRE_DOCUMENT_TYPE.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::category::category_id::CategoryId;
    use crate::domain::category::nested_category::NestedCategoryCreateCommand;
    use crate::domain::genre::aggregate::GenreCreateCommand;

    fn make_genre() -> Genre {
        Genre::create(GenreCreateCommand {
            genre_id: GenreId::new(),
            name: "Action".to_string(),
            categories_props: vec![NestedCategoryCreateCommand {
                category_id: CategoryId::new(),
                name: "Movie".to_string(),
                is_active: true,
                deleted_at: None,
            }],
            is_active: true,
            created_at: Utc::now(),
        })
    }

    #[test]
    fn should_convert_entity_to_document() {
        let genre = make_genre();
        let doc = GenreElasticSearchMapper::to_document(&genre);

        assert_eq!(doc.genre_name, "Action");
        assert!(doc.is_active);
        assert!(doc.deleted_at.is_none());
        assert_eq!(doc.doc_type, "Genre");
        assert_eq!(doc.categories.len(), 1);
        assert_eq!(doc.categories[0].category_name, "Movie");
        assert!(doc.categories[0].is_active);
        assert!(!doc.categories[0].is_deleted);
    }

    #[test]
    fn should_convert_document_to_entity() {
        let category_id = CategoryId::new();
        let doc = GenreDocument {
            genre_name: "Action".to_string(),
            categories: vec![NestedCategoryDocument {
                category_id: category_id.to_string(),
                category_name: "Movie".to_string(),
                is_active: true,
                deleted_at: None,
                is_deleted: false,
            }],
            is_active: true,
            created_at: Utc::now(),
            deleted_at: None,
            doc_type: GENRE_DOCUMENT_TYPE.to_string(),
        };

        let genre_id = GenreId::new();
        let entity = GenreElasticSearchMapper::to_entity(&genre_id.to_string(), &doc)
            .expect("should map");

        assert_eq!(entity.name(), "Action");
        assert!(entity.is_active());
        assert_eq!(entity.categories().len(), 1);
    }

    #[test]
    fn should_fail_with_invalid_type() {
        let doc = GenreDocument {
            genre_name: "Action".to_string(),
            categories: vec![],
            is_active: true,
            created_at: Utc::now(),
            deleted_at: None,
            doc_type: "Invalid".to_string(),
        };

        let result = GenreElasticSearchMapper::to_entity("some-id", &doc);
        assert!(result.is_err());
    }

    #[test]
    fn should_handle_deleted_nested_category() {
        let category_id = CategoryId::new();
        let deleted_at = Utc::now();
        let doc = GenreDocument {
            genre_name: "Drama".to_string(),
            categories: vec![NestedCategoryDocument {
                category_id: category_id.to_string(),
                category_name: "Series".to_string(),
                is_active: false,
                deleted_at: Some(deleted_at),
                is_deleted: true,
            }],
            is_active: true,
            created_at: Utc::now(),
            deleted_at: None,
            doc_type: GENRE_DOCUMENT_TYPE.to_string(),
        };

        let genre_id = GenreId::new();
        let entity = GenreElasticSearchMapper::to_entity(&genre_id.to_string(), &doc)
            .expect("should map");

        let cat = entity.categories().values().next().unwrap();
        assert!(!cat.is_active());
        assert!(cat.deleted_at().is_some());
    }
}
