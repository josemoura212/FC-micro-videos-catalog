use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domain::category::aggregate::Category;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CategoryOutput {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

pub struct CategoryOutputMapper;

impl CategoryOutputMapper {
    #[must_use]
    pub fn to_output(entity: &Category) -> CategoryOutput {
        CategoryOutput {
            id: entity.category_id().to_string(),
            name: entity.name().to_string(),
            description: entity.description().map(String::from),
            is_active: entity.is_active(),
            created_at: entity.created_at(),
            deleted_at: entity.deleted_at(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::category::aggregate::CategoryCreateCommand;
    use crate::domain::category::category_id::CategoryId;

    use super::*;

    #[test]
    fn should_convert_category_to_output() {
        let category = Category::create(CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            description: Some("some description".to_string()),
            is_active: true,
            created_at: Utc::now(),
        });

        let output = CategoryOutputMapper::to_output(&category);
        assert_eq!(output.id, category.category_id().to_string());
        assert_eq!(output.name, "Movie");
        assert_eq!(output.description, Some("some description".to_string()));
        assert!(output.is_active);
        assert!(output.deleted_at.is_none());
    }
}
