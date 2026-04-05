use std::sync::Arc;

use elasticsearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use elasticsearch::Elasticsearch;
use url::Url;

use crate::config::Config;
use crate::infrastructure::elasticsearch::cast_member_es_repository::CastMemberElasticSearchRepository;
use crate::infrastructure::elasticsearch::category_es_repository::CategoryElasticSearchRepository;
use crate::infrastructure::elasticsearch::genre_es_repository::GenreElasticSearchRepository;
use crate::infrastructure::elasticsearch::video_es_repository::VideoElasticSearchRepository;

#[derive(Clone)]
pub struct AppState {
    pub category_repo: Arc<CategoryElasticSearchRepository>,
    pub genre_repo: Arc<GenreElasticSearchRepository>,
    pub cast_member_repo: Arc<CastMemberElasticSearchRepository>,
    pub video_repo: Arc<VideoElasticSearchRepository>,
}

impl AppState {
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let client = create_es_client(&config.elastic_search_host);
        let index = config.elastic_search_index.clone();

        Self {
            category_repo: Arc::new(CategoryElasticSearchRepository::new(
                client.clone(),
                index.clone(),
            )),
            genre_repo: Arc::new(GenreElasticSearchRepository::new(
                client.clone(),
                index.clone(),
            )),
            cast_member_repo: Arc::new(CastMemberElasticSearchRepository::new(
                client.clone(),
                index.clone(),
            )),
            video_repo: Arc::new(VideoElasticSearchRepository::new(client, index)),
        }
    }
}

fn create_es_client(host: &str) -> Elasticsearch {
    let url = Url::parse(host).expect("invalid ELASTIC_SEARCH_HOST URL");
    let pool = SingleNodeConnectionPool::new(url);
    let transport = TransportBuilder::new(pool)
        .disable_proxy()
        .build()
        .expect("failed to build Elasticsearch transport");
    Elasticsearch::new(transport)
}
