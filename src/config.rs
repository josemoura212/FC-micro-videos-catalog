pub struct Config {
    pub elastic_search_host: String,
    pub elastic_search_index: String,
    pub port: u16,
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
        }
    }
}
