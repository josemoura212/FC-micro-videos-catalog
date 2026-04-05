use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::application::category::delete_category::{DeleteCategoryInput, DeleteCategoryUseCase};
use crate::application::category::save_category::{SaveCategoryInput, SaveCategoryUseCase};
use crate::domain::category::category_repository::ICategoryRepository;

use super::cdc::{CdcOperation, CdcPayload};
use super::consumer::ConsumerError;

#[derive(Debug, Clone, Deserialize)]
pub struct CategoryCdcData {
    pub category_id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: MysqlBool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(from = "u8")]
pub struct MysqlBool(bool);

impl From<u8> for MysqlBool {
    fn from(value: u8) -> Self {
        Self(value != 0)
    }
}

impl MysqlBool {
    #[must_use]
    pub const fn as_bool(self) -> bool {
        self.0
    }
}

pub struct CategoryCdcHandler<R: ICategoryRepository> {
    save_use_case: SaveCategoryUseCase<R>,
    delete_use_case: DeleteCategoryUseCase<R>,
}

impl<R: ICategoryRepository> CategoryCdcHandler<R>
where
    R: Clone,
{
    #[must_use]
    pub fn new(repo: R) -> Self {
        Self {
            save_use_case: SaveCategoryUseCase::new(repo.clone()),
            delete_use_case: DeleteCategoryUseCase::new(repo),
        }
    }

    /// # Errors
    /// Returns `ConsumerError` on deserialization or use case failure.
    pub async fn handle(&self, value: &[u8], topic: &str) -> Result<(), ConsumerError> {
        let payload: CdcPayload<CategoryCdcData> = serde_json::from_slice(value)
            .map_err(|e| ConsumerError::deserialization(e.to_string()))?;

        match payload.op {
            CdcOperation::Read | CdcOperation::Create | CdcOperation::Update => {
                let data = payload.after.ok_or_else(|| {
                    ConsumerError::missing_after(topic, payload.op.to_string())
                })?;
                self.handle_upsert(data).await
            }
            CdcOperation::Delete => {
                let data = payload.before.ok_or_else(|| {
                    ConsumerError::missing_before(topic, payload.op.to_string())
                })?;
                self.handle_delete(data).await
            }
        }
    }

    async fn handle_upsert(&self, data: CategoryCdcData) -> Result<(), ConsumerError> {
        let input = SaveCategoryInput {
            category_id: data.category_id,
            name: data.name,
            description: data.description,
            is_active: data.is_active.as_bool(),
            created_at: data.created_at,
        };

        self.save_use_case
            .execute(input)
            .await
            .map_err(|e| ConsumerError::handler(e.to_string()))?;

        Ok(())
    }

    async fn handle_delete(&self, data: CategoryCdcData) -> Result<(), ConsumerError> {
        let input = DeleteCategoryInput {
            id: data.category_id,
        };

        self.delete_use_case
            .execute(input)
            .await
            .map_err(|e| ConsumerError::handler(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::category::aggregate::{Category, CategoryCreateCommand};
    use crate::domain::category::category_id::CategoryId;
    use crate::infrastructure::in_memory::category_in_memory_repository::CategoryInMemoryRepository;

    fn make_handler() -> CategoryCdcHandler<CategoryInMemoryRepository> {
        let repo = CategoryInMemoryRepository::new();
        CategoryCdcHandler::new(repo)
    }

    fn make_create_payload(category_id: &str) -> String {
        let ts = Utc::now().to_rfc3339();
        format!(
            r#"{{"op":"c","before":null,"after":{{"category_id":"{category_id}","name":"Movie","description":"A movie category","is_active":1,"created_at":"{ts}"}}}}"#,
        )
    }

    fn make_update_payload(category_id: &str) -> String {
        let ts = Utc::now().to_rfc3339();
        format!(
            r#"{{"op":"u","before":{{"category_id":"{category_id}","name":"Movie","description":null,"is_active":1,"created_at":"{ts}"}},"after":{{"category_id":"{category_id}","name":"Documentary","description":"Updated","is_active":0,"created_at":"{ts}"}}}}"#,
        )
    }

    fn make_delete_payload(category_id: &str) -> String {
        let ts = Utc::now().to_rfc3339();
        format!(
            r#"{{"op":"d","before":{{"category_id":"{category_id}","name":"Movie","description":null,"is_active":1,"created_at":"{ts}"}},"after":null}}"#,
        )
    }

    #[tokio::test]
    async fn should_handle_create_event() {
        let handler = make_handler();
        let category_id = CategoryId::new().to_string();
        let payload = make_create_payload(&category_id);

        let result = handler.handle(payload.as_bytes(), "categories").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_handle_update_event() {
        let repo = CategoryInMemoryRepository::new();
        let category = Category::create(CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            description: None,
            is_active: true,
            created_at: Utc::now(),
        });
        repo.insert(&category).await.expect("insert");

        let handler = CategoryCdcHandler::new(repo);
        let payload = make_update_payload(&category.category_id().to_string());

        let result = handler.handle(payload.as_bytes(), "categories").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_handle_delete_event() {
        let repo = CategoryInMemoryRepository::new();
        let category = Category::create(CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            description: None,
            is_active: true,
            created_at: Utc::now(),
        });
        repo.insert(&category).await.expect("insert");

        let handler = CategoryCdcHandler::new(repo);
        let payload = make_delete_payload(&category.category_id().to_string());

        let result = handler.handle(payload.as_bytes(), "categories").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_fail_with_invalid_json() {
        let handler = make_handler();
        let result = handler.handle(b"not json", "categories").await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.is_deserialization());
    }

    #[tokio::test]
    async fn should_fail_when_after_is_missing_on_create() {
        let payload = r#"{"op":"c","before":null,"after":null}"#;
        let handler = make_handler();
        let result = handler.handle(payload.as_bytes(), "categories").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing 'after'"));
    }

    #[tokio::test]
    async fn should_fail_when_before_is_missing_on_delete() {
        let payload = r#"{"op":"d","before":null,"after":null}"#;
        let handler = make_handler();
        let result = handler.handle(payload.as_bytes(), "categories").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("missing 'before'")
        );
    }

    #[test]
    fn should_deserialize_mysql_bool_true() {
        let json = r#"1"#;
        let val: MysqlBool = serde_json::from_str(json).expect("deserialize");
        assert!(val.as_bool());
    }

    #[test]
    fn should_deserialize_mysql_bool_false() {
        let json = r#"0"#;
        let val: MysqlBool = serde_json::from_str(json).expect("deserialize");
        assert!(!val.as_bool());
    }
}
