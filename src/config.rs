pub struct Config {
    pub elastic_search_host: String,
    pub elastic_search_index: String,
    pub port: u16,
    pub kafka_brokers: String,
    pub kafka_connect_prefix: String,
    pub schema_registry_url: String,
}

impl Config {
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            elastic_search_host: std::env::var("ELASTIC_SEARCH_HOST")
                .unwrap_or_else(|_| "http://localhost:9200".to_string()),
            elastic_search_index: std::env::var("ELASTIC_SEARCH_INDEX")
                .unwrap_or_else(|_| "catalog".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            kafka_brokers: std::env::var("KAFKA_BROKERS")
                .unwrap_or_else(|_| "kafka:29092".to_string()),
            kafka_connect_prefix: std::env::var("KAFKA_CONNECT_PREFIX")
                .unwrap_or_else(|_| "mysql.micro_videos".to_string()),
            schema_registry_url: std::env::var("SCHEMA_REGISTRY_URL")
                .unwrap_or_else(|_| "http://schema-registry:8081".to_string()),
        }
    }
}
