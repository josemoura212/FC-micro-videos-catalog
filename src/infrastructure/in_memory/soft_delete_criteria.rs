use crate::domain::category::aggregate::Category;
use crate::domain::shared::criteria::Criteria;

pub struct SoftDeleteInMemoryCriteria;

impl Criteria<Vec<Category>> for SoftDeleteInMemoryCriteria {
    fn apply(&self, context: Vec<Category>) -> Vec<Category> {
        context
            .into_iter()
            .filter(|item| item.deleted_at().is_none())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::category::aggregate::CategoryCreateCommand;
    use crate::domain::category::category_id::CategoryId;

    #[test]
    fn should_filter_deleted_categories() {
        let mut cat1 = Category::create(CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Active".to_string(),
            description: None,
            is_active: true,
            created_at: Utc::now(),
        });
        let cat2 = Category::create(CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Not deleted".to_string(),
            description: None,
            is_active: true,
            created_at: Utc::now(),
        });

        cat1.mark_as_deleted();

        let criteria = SoftDeleteInMemoryCriteria;
        let result = criteria.apply(vec![cat1, cat2]);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name(), "Not deleted");
    }
}
