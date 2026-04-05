use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers_modules::kafka::Kafka;
use tokio::sync::{OnceCell, Semaphore, SemaphorePermit};

struct SharedKafkaContainer {
    host_port: u16,
    _container: ContainerAsync<Kafka>,
}

static SHARED_KAFKA: OnceCell<SharedKafkaContainer> = OnceCell::const_new();
static KAFKA_CONCURRENCY: Semaphore = Semaphore::const_new(1);

async fn shared_kafka() -> &'static SharedKafkaContainer {
    SHARED_KAFKA
        .get_or_init(|| async {
            let container = Kafka::default()
                .start()
                .await
                .expect("Failed to start Kafka container");

            let host_port = container
                .get_host_port_ipv4(9093)
                .await
                .expect("Failed to get host port");

            SharedKafkaContainer {
                host_port,
                _container: container,
            }
        })
        .await
}

pub struct KafkaTestHelper {
    brokers: String,
    _permit: SemaphorePermit<'static>,
}

impl KafkaTestHelper {
    /// Start or reuse the shared Kafka container.
    ///
    /// # Errors
    /// Returns error if container fails to start.
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let permit = KAFKA_CONCURRENCY
            .acquire()
            .await
            .map_err(|e| format!("Failed to acquire semaphore: {e}"))?;

        let shared = shared_kafka().await;
        let brokers = format!("127.0.0.1:{}", shared.host_port);

        Ok(Self {
            brokers,
            _permit: permit,
        })
    }

    #[must_use]
    pub fn brokers(&self) -> &str {
        &self.brokers
    }

    /// Create a Kafka topic.
    ///
    /// # Errors
    /// Returns error if topic creation fails.
    pub async fn create_topic(&self, topic: &str) -> Result<(), Box<dyn std::error::Error>> {
        let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
            .set("bootstrap.servers", &self.brokers)
            .create()?;

        let new_topic = NewTopic::new(topic, 1, TopicReplication::Fixed(1));
        admin
            .create_topics(&[new_topic], &AdminOptions::new())
            .await?;
        Ok(())
    }

    /// Delete a Kafka topic.
    ///
    /// # Errors
    /// Returns error if topic deletion fails.
    pub async fn delete_topic(&self, topic: &str) -> Result<(), Box<dyn std::error::Error>> {
        let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
            .set("bootstrap.servers", &self.brokers)
            .create()?;

        admin
            .delete_topics(&[topic], &AdminOptions::new())
            .await?;
        Ok(())
    }

    /// Build a Kafka producer connected to the test broker.
    ///
    /// # Errors
    /// Returns error if producer creation fails.
    pub fn build_producer(&self) -> Result<FutureProducer, rdkafka::error::KafkaError> {
        ClientConfig::new()
            .set("bootstrap.servers", &self.brokers)
            .set("message.timeout.ms", "5000")
            .create()
    }

    /// Send a JSON message to a topic.
    ///
    /// # Errors
    /// Returns error if producer creation or send fails.
    pub async fn send_json(
        &self,
        topic: &str,
        payload: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let producer = self.build_producer()?;
        producer
            .send(
                FutureRecord::to(topic)
                    .payload(payload)
                    .key(""),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(e, _)| format!("Send failed: {e}"))?;
        Ok(())
    }

    /// Send a tombstone (null value) message to a topic.
    ///
    /// # Errors
    /// Returns error if send fails.
    pub async fn send_tombstone(&self, topic: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Tombstone = message with null payload; send empty bytes as closest equivalent
        self.send_json(topic, "").await
    }
}
