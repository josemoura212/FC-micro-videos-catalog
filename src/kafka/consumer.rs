use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use std::fmt;
use std::sync::Arc;
use tracing::{error, info, warn};

use super::category_consumer::{CategoryConsumer, CategoryConsumerError};
use super::retry::{with_retry_and_dlq, ErrorKind, RetryConfig};
use crate::domain::category::category_repository::ICategoryRepository;

/// Builds an rdkafka `StreamConsumer` for CDC consumption.
///
/// # Errors
/// Returns error if Kafka configuration is invalid.
pub fn build_consumer(
    brokers: &str,
    group_id: &str,
) -> Result<StreamConsumer, rdkafka::error::KafkaError> {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .set("session.timeout.ms", "10000")
        .create()
}

/// Classifica erros do consumer em retriáveis ou não-retriáveis.
fn classify_error(err: &CategoryConsumerError) -> ErrorKind {
    match err {
        // Dados inválidos → não faz sentido retentar, vai para DLQ
        CategoryConsumerError::Deserialization(_)
        | CategoryConsumerError::InvalidDate(_)
        | CategoryConsumerError::MissingField => ErrorKind::NonRetriable,
        // Falhas de use case (ex: ES fora do ar) → retriável
        CategoryConsumerError::UseCase(_) => ErrorKind::Retriable,
    }
}

/// Runs the category CDC consumer loop with retry and dead-letter queue.
/// Subscribes to `topic` and dispatches each message to `CategoryConsumer`.
///
/// - Erros de desserialização/validação → DLQ imediato
/// - Erros de infraestrutura → retry com backoff [1s, 3s, 9s]
///
/// # Errors
/// Returns error if Kafka subscription fails.
pub async fn run_category_consumer<
    SR: ICategoryRepository + Send + Sync + 'static,
    DR: ICategoryRepository + Send + Sync + 'static,
>(
    consumer: StreamConsumer,
    topic: &str,
    brokers: &str,
    category_consumer: CategoryConsumer<SR, DR>,
    retry_config: RetryConfig,
) -> Result<(), rdkafka::error::KafkaError> {
    consumer.subscribe(&[topic])?;
    info!("[KafkaConsumer] Subscribed to topic: {topic}");

    let category_consumer = Arc::new(category_consumer);

    loop {
        match consumer.recv().await {
            Err(e) => {
                warn!("[KafkaConsumer] Kafka error: {e}");
            }
            Ok(msg) => {
                let payload_bytes = msg.payload().unwrap_or(&[]);
                let payload_owned = payload_bytes.to_vec();
                let topic_str = msg.topic().to_string();
                let handler = Arc::clone(&category_consumer);

                let result = with_retry_and_dlq(
                    || {
                        let payload = payload_owned.clone();
                        let h = Arc::clone(&handler);
                        async move {
                            h.handle(if payload.is_empty() { None } else { Some(&payload) })
                                .await
                        }
                    },
                    classify_error,
                    &payload_owned,
                    &topic_str,
                    brokers,
                    &retry_config,
                )
                .await;

                match result {
                    Ok(()) => {
                        info!(
                            "[KafkaConsumer] Processed — topic: {}, partition: {}, offset: {}",
                            msg.topic(),
                            msg.partition(),
                            msg.offset()
                        );
                    }
                    Err(e) => {
                        error!("[KafkaConsumer] Failed to send to DLQ: {e}");
                    }
                }
            }
        }
    }
}

// ─── Error type (BurntSushi pattern: pub struct + private enum) ──────────────

#[derive(Debug)]
pub struct ConsumerError {
    inner: ConsumerErrorInner,
}

#[derive(Debug)]
enum ConsumerErrorInner {
    Deserialization { message: String },
    Handler { message: String },
    Tombstone { topic: String },
    MissingAfterField { topic: String, op: String },
    MissingBeforeField { topic: String, op: String },
}

impl ConsumerError {
    #[must_use]
    pub fn deserialization(message: impl Into<String>) -> Self {
        Self {
            inner: ConsumerErrorInner::Deserialization {
                message: message.into(),
            },
        }
    }

    #[must_use]
    pub fn handler(message: impl Into<String>) -> Self {
        Self {
            inner: ConsumerErrorInner::Handler {
                message: message.into(),
            },
        }
    }

    #[must_use]
    pub fn tombstone(topic: impl Into<String>) -> Self {
        Self {
            inner: ConsumerErrorInner::Tombstone {
                topic: topic.into(),
            },
        }
    }

    #[must_use]
    pub fn missing_after(topic: impl Into<String>, op: impl Into<String>) -> Self {
        Self {
            inner: ConsumerErrorInner::MissingAfterField {
                topic: topic.into(),
                op: op.into(),
            },
        }
    }

    #[must_use]
    pub fn missing_before(topic: impl Into<String>, op: impl Into<String>) -> Self {
        Self {
            inner: ConsumerErrorInner::MissingBeforeField {
                topic: topic.into(),
                op: op.into(),
            },
        }
    }

    #[must_use]
    pub fn is_tombstone(&self) -> bool {
        matches!(self.inner, ConsumerErrorInner::Tombstone { .. })
    }

    #[must_use]
    pub fn is_deserialization(&self) -> bool {
        matches!(self.inner, ConsumerErrorInner::Deserialization { .. })
    }

    #[must_use]
    pub fn is_handler(&self) -> bool {
        matches!(self.inner, ConsumerErrorInner::Handler { .. })
    }
}

impl fmt::Display for ConsumerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            ConsumerErrorInner::Deserialization { message } => {
                write!(f, "deserialization error: {message}")
            }
            ConsumerErrorInner::Handler { message } => {
                write!(f, "handler error: {message}")
            }
            ConsumerErrorInner::Tombstone { topic } => {
                write!(f, "tombstone message on topic {topic} (skipped)")
            }
            ConsumerErrorInner::MissingAfterField { topic, op } => {
                write!(f, "missing 'after' field for op={op} on topic {topic}")
            }
            ConsumerErrorInner::MissingBeforeField { topic, op } => {
                write!(f, "missing 'before' field for op={op} on topic {topic}")
            }
        }
    }
}

impl std::error::Error for ConsumerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_deserialization_error() {
        let err = ConsumerError::deserialization("bad json");
        assert!(err.is_deserialization());
        assert!(!err.is_tombstone());
        assert!(!err.is_handler());
        assert!(err.to_string().contains("bad json"));
    }

    #[test]
    fn should_create_handler_error() {
        let err = ConsumerError::handler("use case failed");
        assert!(err.is_handler());
        assert!(err.to_string().contains("use case failed"));
    }

    #[test]
    fn should_create_tombstone_error() {
        let err = ConsumerError::tombstone("categories");
        assert!(err.is_tombstone());
        assert!(err.to_string().contains("categories"));
    }

    #[test]
    fn should_create_missing_after_error() {
        let err = ConsumerError::missing_after("categories", "c");
        assert!(err.to_string().contains("missing 'after' field"));
        assert!(err.to_string().contains("op=c"));
    }

    #[test]
    fn should_create_missing_before_error() {
        let err = ConsumerError::missing_before("categories", "d");
        assert!(err.to_string().contains("missing 'before' field"));
        assert!(err.to_string().contains("op=d"));
    }
}
