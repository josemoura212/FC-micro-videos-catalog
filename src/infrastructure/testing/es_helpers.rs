use elasticsearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use elasticsearch::indices::IndicesCreateParts;
use elasticsearch::Elasticsearch;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers_modules::elastic_search::ElasticSearch;
use url::Url;

use crate::infrastructure::elasticsearch::es_mapping::es_mapping;

pub struct EsTestHelper {
    pub client: Elasticsearch,
    pub index: String,
    _container: ContainerAsync<ElasticSearch>,
}

impl EsTestHelper {
    /// # Errors
    /// Returns error if container fails to start or index creation fails.
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let container = ElasticSearch::default()
            .start()
            .await
            .map_err(|e| format!("Failed to start ES container: {e}"))?;

        let host_port = container
            .get_host_port_ipv4(9200)
            .await
            .map_err(|e| format!("Failed to get host port: {e}"))?;

        let url_str = format!("http://127.0.0.1:{host_port}");
        let parsed_url =
            Url::parse(&url_str).map_err(|e| format!("Failed to parse URL: {e}"))?;
        let pool = SingleNodeConnectionPool::new(parsed_url);
        let transport = TransportBuilder::new(pool)
            .disable_proxy()
            .build()
            .map_err(|e| format!("Failed to build transport: {e}"))?;
        let client = Elasticsearch::new(transport);

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
            _container: container,
        })
    }
}
