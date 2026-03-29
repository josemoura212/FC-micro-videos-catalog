use elasticsearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use elasticsearch::indices::IndicesCreateParts;
use elasticsearch::Elasticsearch;
use testcontainers::core::{ImageExt, Mount};
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers_modules::elastic_search::ElasticSearch;
use tokio::sync::{OnceCell, Semaphore, SemaphorePermit};
use url::Url;

use crate::infrastructure::elasticsearch::es_mapping::es_mapping;

struct SharedEsContainer {
    client: Elasticsearch,
    _container: ContainerAsync<ElasticSearch>,
}

static SHARED_ES: OnceCell<SharedEsContainer> = OnceCell::const_new();
static ES_CONCURRENCY: Semaphore = Semaphore::const_new(1);

async fn shared_es() -> &'static SharedEsContainer {
    SHARED_ES
        .get_or_init(|| async {
            let container = ElasticSearch::default()
                .with_tag("7.17.7")
                .with_env_var("ES_JAVA_OPTS", "-Xms1g -Xmx1g")
                .with_mount(Mount::tmpfs_mount("/usr/share/elasticsearch/data"))
                .start()
                .await
                .expect("Failed to start ES container");

            let host_port = container
                .get_host_port_ipv4(9200)
                .await
                .expect("Failed to get host port");

            let url_str = format!("http://127.0.0.1:{host_port}");
            let parsed_url = Url::parse(&url_str).expect("Failed to parse URL");
            let pool = SingleNodeConnectionPool::new(parsed_url);
            let transport = TransportBuilder::new(pool)
                .disable_proxy()
                .build()
                .expect("Failed to build transport");

            SharedEsContainer {
                client: Elasticsearch::new(transport),
                _container: container,
            }
        })
        .await
}

pub struct EsTestHelper {
    client: Elasticsearch,
    index: String,
    _permit: SemaphorePermit<'static>,
}

impl EsTestHelper {
    #[must_use]
    pub fn client(&self) -> Elasticsearch {
        self.client.clone()
    }

    #[must_use]
    pub fn index(&self) -> String {
        self.index.clone()
    }

    /// # Errors
    /// Returns error if container fails to start or index creation fails.
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let permit = ES_CONCURRENCY
            .acquire()
            .await
            .map_err(|e| format!("Failed to acquire semaphore: {e}"))?;

        let shared = shared_es().await;
        let client = shared.client.clone();
        let index = format!("test_es_{}", uuid::Uuid::new_v4().simple());

        client
            .indices()
            .create(IndicesCreateParts::Index(&index))
            .body(es_mapping())
            .send()
            .await
            .map_err(|e| format!("Failed to create index: {e}"))?;

        Ok(Self {
            client,
            index,
            _permit: permit,
        })
    }
}
