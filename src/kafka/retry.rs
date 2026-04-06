use std::future::Future;
use std::time::Duration;

use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use tracing::error;

/// Classifica se um erro deve ser retriado ou enviado direto para a DLQ.
#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    /// Erro permanente — dado inválido ou negócio. Vai para DLQ sem retry.
    NonRetriable,
    /// Erro transitório — infra, timeout, etc. Será retriado.
    Retriable,
}

/// Configuração de retry com backoff escalonado.
pub struct RetryConfig {
    /// Delays em segundos entre tentativas (ex: [1, 3, 9]).
    pub retry_delays: Vec<u64>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            retry_delays: vec![1, 3, 9],
        }
    }
}

impl RetryConfig {
    #[must_use]
    pub fn max_retries(&self) -> usize {
        self.retry_delays.len()
    }
}

/// Produz uma mensagem para o tópico de dead-letter.
///
/// # Errors
/// Retorna erro se a produção falhar.
pub async fn send_to_dlq(
    brokers: &str,
    original_topic: &str,
    payload: &[u8],
    error_message: &str,
) -> Result<(), String> {
    let dlq_topic = format!("{original_topic}-dlq");
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .map_err(|e| format!("Failed to create DLQ producer: {e}"))?;

    let key = error_message.as_bytes();
    producer
        .send(
            FutureRecord::to(&dlq_topic)
                .payload(payload)
                .key(key),
            Duration::from_secs(5),
        )
        .await
        .map_err(|(e, _)| format!("Failed to send to DLQ: {e}"))?;

    Ok(())
}

/// Executa um handler com retry + DLQ.
///
/// - Erros `NonRetriable` → DLQ imediatamente (sem retry).
/// - Erros `Retriable` → tenta até `config.max_retries()` vezes com os delays configurados.
/// - Se esgotar retries → DLQ.
///
/// # Errors
/// Retorna erro apenas se o envio para DLQ falhar.
pub async fn with_retry_and_dlq<F, Fut, E>(
    handler: F,
    classify: impl Fn(&E) -> ErrorKind,
    payload: &[u8],
    topic: &str,
    brokers: &str,
    config: &RetryConfig,
) -> Result<(), String>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<(), E>>,
    E: std::fmt::Display,
{
    let result = handler().await;

    if result.is_ok() {
        return Ok(());
    }

    let err = result.unwrap_err();

    match classify(&err) {
        ErrorKind::NonRetriable => {
            error!("[Retry] Non-retriable error — sending to DLQ: {err}");
            send_to_dlq(brokers, topic, payload, &err.to_string()).await?;
            return Ok(());
        }
        ErrorKind::Retriable => {}
    }

    for (attempt, &delay_secs) in config.retry_delays.iter().enumerate() {
        error!(
            "[Retry] Attempt {}/{} failed: {err}. Retrying in {delay_secs}s...",
            attempt + 1,
            config.max_retries()
        );
        tokio::time::sleep(Duration::from_secs(delay_secs)).await;

        match handler().await {
            Ok(()) => return Ok(()),
            Err(e) => {
                if matches!(classify(&e), ErrorKind::NonRetriable) {
                    error!("[Retry] Non-retriable error on retry — sending to DLQ: {e}");
                    send_to_dlq(brokers, topic, payload, &e.to_string()).await?;
                    return Ok(());
                }
                // Continue para o próximo retry
            }
        }
    }

    // Esgotou retries → DLQ
    error!("[Retry] Exhausted retries — sending to DLQ: {err}");
    send_to_dlq(brokers, topic, payload, &err.to_string()).await?;
    Ok(())
}
