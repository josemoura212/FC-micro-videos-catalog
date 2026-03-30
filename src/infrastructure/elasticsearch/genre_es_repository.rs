use async_trait::async_trait;
use elasticsearch::http::request::JsonBody;
use elasticsearch::params::Refresh;
use elasticsearch::{
    BulkParts, DeleteByQueryParts, Elasticsearch, IndexParts, SearchParts, UpdateByQueryParts,
};
use serde_json::{json, Value};

use crate::domain::genre::aggregate::Genre;
use crate::domain::genre::genre_id::GenreId;
use crate::domain::genre::genre_repository::IGenreRepository;
use crate::domain::shared::errors::NotFoundError;
use crate::domain::shared::repository::{ExistsByIdResult, FindByIdsResult, SortOrder};
use crate::domain::shared::value_object::UuidVo;

use super::genre_mapper::{GenreDocument, GenreElasticSearchMapper, GENRE_DOCUMENT_TYPE};

#[derive(Debug, thiserror::Error)]
pub enum GenreEsRepositoryError {
    #[error(transparent)]
    NotFound(#[from] NotFoundError),
    #[error("elasticsearch error: {0}")]
    Elasticsearch(String),
    #[error("mapping error: {0}")]
    Mapping(String),
}

pub struct GenreElasticSearchRepository {
    client: Elasticsearch,
    index: String,
    soft_delete_scope: bool,
}

impl GenreElasticSearchRepository {
    #[must_use]
    pub const fn new(client: Elasticsearch, index: String) -> Self {
        Self {
            client,
            index,
            soft_delete_scope: false,
        }
    }

    fn build_filter_query(
        &self,
        genre_id: Option<&GenreId>,
        is_active: Option<bool>,
    ) -> Value {
        let mut must = vec![json!({"match": {"type": GENRE_DOCUMENT_TYPE}})];

        if let Some(id) = genre_id {
            must.push(json!({"match": {"_id": id.to_string()}}));
        }

        if let Some(active) = is_active {
            must.push(json!({"match": {"is_active": active}}));
        }

        let mut query = json!({"bool": {"must": must}});

        if self.soft_delete_scope {
            query["bool"]["must_not"] = json!([{"exists": {"field": "deleted_at"}}]);
        }

        query
    }

    fn parse_hits(body: &Value) -> Vec<(String, GenreDocument)> {
        body["hits"]["hits"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|hit| {
                let id = hit["_id"].as_str()?;
                let source: GenreDocument =
                    serde_json::from_value(hit["_source"].clone()).ok()?;
                Some((id.to_string(), source))
            })
            .collect()
    }
}

impl crate::domain::shared::criteria::ScopedRepository for GenreElasticSearchRepository {
    fn ignore_soft_deleted(&mut self) -> &mut Self {
        self.soft_delete_scope = true;
        self
    }

    fn clear_scopes(&mut self) -> &mut Self {
        self.soft_delete_scope = false;
        self
    }
}

#[async_trait]
impl IGenreRepository for GenreElasticSearchRepository {
    type Error = GenreEsRepositoryError;

    fn sortable_fields(&self) -> &[&str] {
        &["name", "created_at"]
    }

    async fn insert(&self, entity: &Genre) -> Result<(), Self::Error> {
        let doc = GenreElasticSearchMapper::to_document(entity);
        let body = serde_json::to_value(&doc)
            .map_err(|e| GenreEsRepositoryError::Mapping(e.to_string()))?;

        self.client
            .index(IndexParts::IndexId(
                &self.index,
                &entity.genre_id().to_string(),
            ))
            .body(body)
            .refresh(Refresh::True)
            .send()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        Ok(())
    }

    async fn bulk_insert(&self, entities: &[Genre]) -> Result<(), Self::Error> {
        let mut body: Vec<JsonBody<Value>> = Vec::with_capacity(entities.len() * 2);

        for entity in entities {
            let doc = GenreElasticSearchMapper::to_document(entity);
            body.push(
                json!({"index": {"_id": entity.genre_id().to_string()}}).into(),
            );
            body.push(
                serde_json::to_value(&doc)
                    .map_err(|e| GenreEsRepositoryError::Mapping(e.to_string()))?
                    .into(),
            );
        }

        self.client
            .bulk(BulkParts::Index(&self.index))
            .body(body)
            .refresh(Refresh::True)
            .send()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &GenreId,
    ) -> Result<Option<Genre>, Self::Error> {
        let query = self.build_filter_query(Some(id), None);

        let response = self
            .client
            .search(SearchParts::Index(&[&self.index]))
            .body(json!({"query": query}))
            .send()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        let hits = Self::parse_hits(&body);

        match hits.first() {
            Some((doc_id, doc)) => {
                let entity = GenreElasticSearchMapper::to_entity(doc_id, doc)
                    .map_err(|e| GenreEsRepositoryError::Mapping(e.to_string()))?;
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }

    async fn find_one_by(
        &self,
        genre_id: Option<&GenreId>,
        is_active: Option<bool>,
    ) -> Result<Option<Genre>, Self::Error> {
        let query = self.build_filter_query(genre_id, is_active);

        let response = self
            .client
            .search(SearchParts::Index(&[&self.index]))
            .body(json!({"query": query}))
            .send()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        let hits = Self::parse_hits(&body);

        match hits.first() {
            Some((doc_id, doc)) => {
                let entity = GenreElasticSearchMapper::to_entity(doc_id, doc)
                    .map_err(|e| GenreEsRepositoryError::Mapping(e.to_string()))?;
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }

    async fn find_by(
        &self,
        genre_id: Option<&GenreId>,
        is_active: Option<bool>,
        order: Option<&SortOrder>,
    ) -> Result<Vec<Genre>, Self::Error> {
        let query = self.build_filter_query(genre_id, is_active);

        let sortable_map = [("name", "genre_name"), ("created_at", "created_at")];
        let sort = order.and_then(|o| {
            sortable_map
                .iter()
                .find(|(k, _)| *k == o.field)
                .map(|(_, v)| json!([{(*v): o.direction.as_str()}]))
        });

        let mut body = json!({"query": query});
        if let Some(s) = sort {
            body["sort"] = s;
        }

        let response = self
            .client
            .search(SearchParts::Index(&[&self.index]))
            .body(body)
            .send()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        let hits = Self::parse_hits(&body);

        hits.iter()
            .map(|(doc_id, doc)| {
                GenreElasticSearchMapper::to_entity(doc_id, doc)
                    .map_err(|e| GenreEsRepositoryError::Mapping(e.to_string()))
            })
            .collect()
    }

    async fn find_all(&self) -> Result<Vec<Genre>, Self::Error> {
        let mut query =
            json!({"bool": {"must": [{"match": {"type": GENRE_DOCUMENT_TYPE}}]}});

        if self.soft_delete_scope {
            query["bool"]["must_not"] = json!([{"exists": {"field": "deleted_at"}}]);
        }

        let response = self
            .client
            .search(SearchParts::Index(&[&self.index]))
            .body(json!({"query": query}))
            .send()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        let hits = Self::parse_hits(&body);

        hits.iter()
            .map(|(doc_id, doc)| {
                GenreElasticSearchMapper::to_entity(doc_id, doc)
                    .map_err(|e| GenreEsRepositoryError::Mapping(e.to_string()))
            })
            .collect()
    }

    async fn find_by_ids(
        &self,
        ids: &[GenreId],
    ) -> Result<FindByIdsResult<Genre>, Self::Error> {
        let id_strings: Vec<String> = ids.iter().map(ToString::to_string).collect();

        let mut query = json!({
            "bool": {
                "must": [
                    {"ids": {"values": id_strings}},
                    {"match": {"type": GENRE_DOCUMENT_TYPE}}
                ]
            }
        });

        if self.soft_delete_scope {
            query["bool"]["must_not"] = json!([{"exists": {"field": "deleted_at"}}]);
        }

        let response = self
            .client
            .search(SearchParts::Index(&[&self.index]))
            .body(json!({"query": query}))
            .send()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        let hits = Self::parse_hits(&body);
        let found_ids: Vec<String> = hits.iter().map(|(id, _)| id.clone()).collect();

        let exists: Result<Vec<Genre>, _> = hits
            .iter()
            .map(|(doc_id, doc)| {
                GenreElasticSearchMapper::to_entity(doc_id, doc)
                    .map_err(|e| GenreEsRepositoryError::Mapping(e.to_string()))
            })
            .collect();

        let not_exists: Vec<UuidVo> = ids
            .iter()
            .filter(|id| !found_ids.contains(&id.to_string()))
            .map(|id| id.inner().clone())
            .collect();

        Ok(FindByIdsResult {
            exists: exists?,
            not_exists,
        })
    }

    async fn exists_by_id(
        &self,
        ids: &[GenreId],
    ) -> Result<ExistsByIdResult, Self::Error> {
        let id_strings: Vec<String> = ids.iter().map(ToString::to_string).collect();

        let query = json!({
            "bool": {
                "must": [
                    {"ids": {"values": id_strings}},
                    {"match": {"type": GENRE_DOCUMENT_TYPE}}
                ]
            }
        });

        let response = self
            .client
            .search(SearchParts::Index(&[&self.index]))
            .body(json!({"query": query, "_source": false}))
            .send()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        let found_ids: Vec<String> = body["hits"]["hits"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|hit| hit["_id"].as_str().map(String::from))
            .collect();

        let mut exists = Vec::new();
        let mut not_exists = Vec::new();

        for id in ids {
            if found_ids.contains(&id.to_string()) {
                exists.push(id.inner().clone());
            } else {
                not_exists.push(id.inner().clone());
            }
        }

        Ok(ExistsByIdResult { exists, not_exists })
    }

    async fn update(&self, entity: &Genre) -> Result<(), Self::Error> {
        let doc = GenreElasticSearchMapper::to_document(entity);
        let params = serde_json::to_value(&doc)
            .map_err(|e| GenreEsRepositoryError::Mapping(e.to_string()))?;

        let script = r"
            ctx._source.genre_name = params.genre_name;
            ctx._source.categories = params.categories;
            ctx._source.is_active = params.is_active;
            ctx._source.created_at = params.created_at;
            ctx._source.deleted_at = params.deleted_at;
        ";

        let response = self
            .client
            .update_by_query(UpdateByQueryParts::Index(&[&self.index]))
            .body(json!({
                "query": {"match": {"_id": entity.genre_id().to_string()}},
                "script": {
                    "source": script,
                    "params": params
                }
            }))
            .refresh(true)
            .send()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        if body["updated"].as_u64().unwrap_or(0) == 0 {
            return Err(NotFoundError::new(
                &entity.genre_id().to_string(),
                "Genre",
            )
            .into());
        }

        Ok(())
    }

    async fn delete(&self, id: &GenreId) -> Result<(), Self::Error> {
        let response = self
            .client
            .delete_by_query(DeleteByQueryParts::Index(&[&self.index]))
            .body(json!({"query": {"match": {"_id": id.to_string()}}}))
            .refresh(true)
            .send()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| GenreEsRepositoryError::Elasticsearch(e.to_string()))?;

        if body["deleted"].as_u64().unwrap_or(0) == 0 {
            return Err(NotFoundError::new(&id.to_string(), "Genre").into());
        }

        Ok(())
    }
}
