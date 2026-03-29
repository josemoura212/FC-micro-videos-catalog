use super::notification::Notification;

#[derive(Debug, Clone, thiserror::Error)]
#[error("Entity Validation Error")]
pub struct EntityValidationError {
    pub notification: Notification,
}

impl EntityValidationError {
    #[must_use] 
    pub const fn new(notification: Notification) -> Self {
        Self { notification }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{entity_name} Not Found using ID {id}")]
pub struct NotFoundError {
    pub id: String,
    pub entity_name: String,
}

impl NotFoundError {
    #[must_use] 
    pub fn new(id: &str, entity_name: &str) -> Self {
        Self {
            id: id.to_string(),
            entity_name: entity_name.to_string(),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Invalid argument: {0}")]
pub struct InvalidArgumentError(pub String);

#[derive(Debug, Clone, thiserror::Error)]
#[error("Load Entity Error")]
pub struct LoadEntityError {
    pub errors: Vec<String>,
}
