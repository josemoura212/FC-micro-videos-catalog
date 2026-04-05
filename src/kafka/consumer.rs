use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use std::fmt;
use tracing::{error, info, warn};

use super::category_consumer::{CategoryConsumer, CategoryConsumerError};
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

/// Runs the category CDC consumer loop.
/// Subscribes to `topic` and dispatches each message to `CategoryConsumer`.
///
/// # Errors
/// Returns error if Kafka subscription fails.
pub async fn run_category_consumer<
    SR: ICategoryRepository + Send + Sync + 'static,
    DR: ICategoryRepository + Send + Sync + 'static,
>(
    consumer: StreamConsumer,
    topic: &str,
    category_consumer: CategoryConsumer<SR, DR>,
) -> Result<(), rdkafka::error::KafkaError> {
    consumer.subscribe(&[topic])?;
    info!("[KafkaConsumer] Subscribed to topic: {topic}");

    loop {
        match consumer.recv().await {
            Err(e) => {
                warn!("[KafkaConsumer] Kafka error: {e}");
            }
            Ok(msg) => {
                let payload = msg.payload();
                match category_consumer.handle(payload).await {
                    Ok(()) => {
                        info!(
                            "[KafkaConsumer] Processed — topic: {}, partition: {}, offset: {}",
                            msg.topic(),
                            msg.partition(),
                            msg.offset()
                        );
                    }
                    Err(CategoryConsumerError::Deserialization(e)) => {
                        error!("[KafkaConsumer] Deserialization error (skip): {e}");
                    }
                    Err(e) => {
                        error!("[KafkaConsumer] Consumer error: {e}");
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
