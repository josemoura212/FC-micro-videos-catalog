use serde::Deserialize;
use tracing::info;

use crate::application::category::delete_category::{
    DeleteCategoryInput, DeleteCategoryUseCase,
};
use crate::application::category::save_category::{SaveCategoryInput, SaveCategoryUseCase};
use crate::domain::category::category_repository::ICategoryRepository;

use super::cdc::{CdcOperation, CdcPayload};

/// Represents the MySQL category row captured via Debezium CDC.
#[derive(Debug, Clone, Deserialize)]
pub struct CategoryCdcData {
    pub category_id: String,
    pub name: String,
    pub description: Option<String>,
    /// MySQL TINYINT(1) — comes as 0 or 1 in CDC payloads
    pub is_active: u8,
    /// ISO 8601 timestamp string
    pub created_at: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CategoryConsumerError {
    #[error("deserialization error: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("use case error: {0}")]
    UseCase(String),
    #[error("invalid date: {0}")]
    InvalidDate(String),
    #[error("missing payload field")]
    MissingField,
}

pub struct CategoryConsumer<SR: ICategoryRepository, DR: ICategoryRepository = SR> {
    save_use_case: SaveCategoryUseCase<SR>,
    delete_use_case: DeleteCategoryUseCase<DR>,
}

impl<SR: ICategoryRepository, DR: ICategoryRepository> CategoryConsumer<SR, DR> {
    #[must_use]
    pub fn new(save_repo: SR, delete_repo: DR) -> Self {
        Self {
            save_use_case: SaveCategoryUseCase::new(save_repo),
            delete_use_case: DeleteCategoryUseCase::new(delete_repo),
        }
    }

    /// Handle a raw CDC message payload (JSON bytes).
    /// Returns Ok(()) even for tombstone events (null value).
    ///
    /// # Errors
    /// Returns error on deserialization failure or use case error.
    pub async fn handle(
        &self,
        value: Option<&[u8]>,
    ) -> Result<(), CategoryConsumerError> {
        let Some(bytes) = value else {
            info!("[CategoryConsumer] Discarding tombstone event");
            return Ok(());
        };

        let payload: CdcPayload<CategoryCdcData> = serde_json::from_slice(bytes)?;

        match payload.op {
            CdcOperation::Read => {
                info!("[CategoryConsumer] Discarding read operation");
            }
            CdcOperation::Create | CdcOperation::Update => {
                let data = payload.after.ok_or(CategoryConsumerError::MissingField)?;

                info!(
                    "[CategoryConsumer] Processing op {:?} - {}",
                    payload.op, data.category_id
                );

                let created_at = chrono::DateTime::parse_from_rfc3339(&data.created_at)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .map_err(|e| CategoryConsumerError::InvalidDate(e.to_string()))?;

                let input = SaveCategoryInput {
                    category_id: data.category_id,
                    name: data.name,
                    description: data.description,
                    is_active: data.is_active != 0,
                    created_at,
                };

                self.save_use_case
                    .execute(input)
                    .await
                    .map_err(|e| CategoryConsumerError::UseCase(e.to_string()))?;
            }
            CdcOperation::Delete => {
                let data = payload.before.ok_or(CategoryConsumerError::MissingField)?;

                info!(
                    "[CategoryConsumer] Processing delete - {}",
                    data.category_id
                );

                self.delete_use_case
                    .execute(DeleteCategoryInput {
                        id: data.category_id,
                    })
                    .await
                    .map_err(|e| CategoryConsumerError::UseCase(e.to_string()))?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::in_memory::category_in_memory_repository::CategoryInMemoryRepository;

    fn make_consumer() -> CategoryConsumer<CategoryInMemoryRepository> {
        CategoryConsumer::new(CategoryInMemoryRepository::new(), CategoryInMemoryRepository::new())
    }

    #[tokio::test]
    async fn should_discard_tombstone_event() {
        let consumer = make_consumer();
        let result = consumer.handle(None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_discard_read_operation() {
        let consumer = make_consumer();
        let payload = serde_json::json!({
            "op": "r",
            "before": null,
            "after": null
        });
        let result = consumer.handle(Some(payload.to_string().as_bytes())).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_create_category_on_create_event() {
        let consumer = make_consumer();
        let payload = serde_json::json!({
            "op": "c",
            "before": null,
            "after": {
                "category_id": "4e9e2e4e-0d1a-4a4b-8c0a-5b0e4e4e4e4e",
                "name": "Movie",
                "description": "Movie category",
                "is_active": 1,
                "created_at": "2024-01-01T00:00:00Z"
            }
        });
        let result = consumer.handle(Some(payload.to_string().as_bytes())).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_return_error_on_invalid_json() {
        let consumer = make_consumer();
        let result = consumer.handle(Some(b"invalid json")).await;
        assert!(matches!(
            result,
            Err(CategoryConsumerError::Deserialization(_))
        ));
    }
}
