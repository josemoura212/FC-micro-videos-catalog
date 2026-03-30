use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domain::genre::aggregate::Genre;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NestedCategoryOutput {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GenreOutput {
    pub id: String,
    pub name: String,
    pub categories: Vec<NestedCategoryOutput>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

pub struct GenreOutputMapper;

impl GenreOutputMapper {
    #[must_use]
    pub fn to_output(entity: &Genre) -> GenreOutput {
        let mut categories: Vec<NestedCategoryOutput> = entity
            .categories()
            .values()
            .map(|nested| NestedCategoryOutput {
                id: nested.category_id().to_string(),
                name: nested.name().to_string(),
                is_active: nested.is_active(),
                deleted_at: nested.deleted_at(),
            })
            .collect();
        categories.sort_by(|a, b| a.name.cmp(&b.name));

        GenreOutput {
            id: entity.genre_id().to_string(),
            name: entity.name().to_string(),
            categories,
            is_active: entity.is_active(),
            created_at: entity.created_at(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::domain::category::category_id::CategoryId;
    use crate::domain::category::nested_category::NestedCategoryCreateCommand;
    use crate::domain::genre::aggregate::GenreCreateCommand;
    use crate::domain::genre::genre_id::GenreId;

    use super::*;

    #[test]
    fn should_convert_genre_to_output() {
        let cat_id = CategoryId::new();
        let genre = Genre::create(GenreCreateCommand {
            genre_id: GenreId::new(),
            name: "Action".to_string(),
            categories_props: vec![NestedCategoryCreateCommand {
                category_id: cat_id.clone(),
                name: "Movie".to_string(),
                is_active: true,
                deleted_at: None,
            }],
            is_active: true,
            created_at: Utc::now(),
        });

        let output = GenreOutputMapper::to_output(&genre);
        assert_eq!(output.id, genre.genre_id().to_string());
        assert_eq!(output.name, "Action");
        assert!(output.is_active);
        assert_eq!(output.categories.len(), 1);
        assert_eq!(output.categories[0].id, cat_id.to_string());
        assert_eq!(output.categories[0].name, "Movie");
        assert!(output.categories[0].is_active);
        assert!(output.categories[0].deleted_at.is_none());
    }

    #[test]
    fn should_convert_genre_without_categories() {
        let genre = Genre::create(GenreCreateCommand {
            genre_id: GenreId::new(),
            name: "Drama".to_string(),
            categories_props: vec![],
            is_active: false,
            created_at: Utc::now(),
        });

        let output = GenreOutputMapper::to_output(&genre);
        assert_eq!(output.name, "Drama");
        assert!(!output.is_active);
        assert!(output.categories.is_empty());
    }
}
