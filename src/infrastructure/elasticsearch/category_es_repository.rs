use async_trait::async_trait;
use elasticsearch::http::request::JsonBody;
use elasticsearch::params::Refresh;
use elasticsearch::{
    BulkParts, DeleteByQueryParts, Elasticsearch, IndexParts, SearchParts, UpdateByQueryParts,
};
use serde_json::{json, Value};

use crate::domain::category::aggregate::Category;
use crate::domain::category::category_id::CategoryId;
use crate::domain::category::category_repository::ICategoryRepository;
use crate::domain::shared::errors::NotFoundError;
use crate::domain::shared::repository::{ExistsByIdResult, FindByIdsResult, SortOrder};
use crate::domain::shared::value_object::UuidVo;

use super::category_mapper::{
    CategoryDocument, CategoryElasticSearchMapper, CATEGORY_DOCUMENT_TYPE,
};

#[derive(Debug, thiserror::Error)]
pub enum EsRepositoryError {
    #[error(transparent)]
    NotFound(#[from] NotFoundError),
    #[error("elasticsearch error: {0}")]
    Elasticsearch(String),
    #[error("mapping error: {0}")]
    Mapping(String),
}

#[derive(Clone)]
pub struct CategoryElasticSearchRepository {
    client: Elasticsearch,
    index: String,
    soft_delete_scope: bool,
}

impl CategoryElasticSearchRepository {
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
        category_id: Option<&CategoryId>,
        is_active: Option<bool>,
    ) -> Value {
        let mut must = vec![json!({"match": {"type": CATEGORY_DOCUMENT_TYPE}})];

        if let Some(id) = category_id {
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

    fn parse_hits(body: &Value) -> Vec<(String, CategoryDocument)> {
        body["hits"]["hits"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|hit| {
                let id = hit["_id"].as_str()?;
                let source: CategoryDocument =
                    serde_json::from_value(hit["_source"].clone()).ok()?;
                Some((id.to_string(), source))
            })
            .collect()
    }
}

impl crate::domain::shared::criteria::ScopedRepository for CategoryElasticSearchRepository {
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
impl ICategoryRepository for CategoryElasticSearchRepository {
    type Error = EsRepositoryError;

    fn sortable_fields(&self) -> &[&str] {
        &["name", "created_at"]
    }

    async fn insert(&self, entity: &Category) -> Result<(), Self::Error> {
        let doc = CategoryElasticSearchMapper::to_document(entity);
        let body = serde_json::to_value(&doc)
            .map_err(|e| EsRepositoryError::Mapping(e.to_string()))?;

        self.client
            .index(IndexParts::IndexId(&self.index, &entity.category_id().to_string()))
            .body(body)
            .refresh(Refresh::True)
            .send()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        Ok(())
    }

    async fn bulk_insert(&self, entities: &[Category]) -> Result<(), Self::Error> {
        let mut body: Vec<JsonBody<Value>> = Vec::with_capacity(entities.len() * 2);

        for entity in entities {
            let doc = CategoryElasticSearchMapper::to_document(entity);
            body.push(
                json!({"index": {"_id": entity.category_id().to_string()}}).into(),
            );
            body.push(
                serde_json::to_value(&doc)
                    .map_err(|e| EsRepositoryError::Mapping(e.to_string()))?
                    .into(),
            );
        }

        self.client
            .bulk(BulkParts::Index(&self.index))
            .body(body)
            .refresh(Refresh::True)
            .send()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &CategoryId) -> Result<Option<Category>, Self::Error> {
        let query = self.build_filter_query(Some(id), None);

        let response = self
            .client
            .search(SearchParts::Index(&[&self.index]))
            .body(json!({"query": query}))
            .send()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let hits = Self::parse_hits(&body);

        match hits.first() {
            Some((doc_id, doc)) => {
                let entity = CategoryElasticSearchMapper::to_entity(doc_id, doc)
                    .map_err(|e| EsRepositoryError::Mapping(e.to_string()))?;
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }

    async fn find_one_by(
        &self,
        category_id: Option<&CategoryId>,
        is_active: Option<bool>,
    ) -> Result<Option<Category>, Self::Error> {
        let query = self.build_filter_query(category_id, is_active);

        let response = self
            .client
            .search(SearchParts::Index(&[&self.index]))
            .body(json!({"query": query}))
            .send()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let hits = Self::parse_hits(&body);

        match hits.first() {
            Some((doc_id, doc)) => {
                let entity = CategoryElasticSearchMapper::to_entity(doc_id, doc)
                    .map_err(|e| EsRepositoryError::Mapping(e.to_string()))?;
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }

    async fn find_by(
        &self,
        category_id: Option<&CategoryId>,
        is_active: Option<bool>,
        order: Option<&SortOrder>,
    ) -> Result<Vec<Category>, Self::Error> {
        let query = self.build_filter_query(category_id, is_active);

        let sortable_map = [("name", "category_name"), ("created_at", "created_at")];
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
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let hits = Self::parse_hits(&body);

        hits.iter()
            .map(|(doc_id, doc)| {
                CategoryElasticSearchMapper::to_entity(doc_id, doc)
                    .map_err(|e| EsRepositoryError::Mapping(e.to_string()))
            })
            .collect()
    }

    async fn find_all(&self) -> Result<Vec<Category>, Self::Error> {
        let mut query = json!({"bool": {"must": [{"match": {"type": CATEGORY_DOCUMENT_TYPE}}]}});

        if self.soft_delete_scope {
            query["bool"]["must_not"] = json!([{"exists": {"field": "deleted_at"}}]);
        }

        let response = self
            .client
            .search(SearchParts::Index(&[&self.index]))
            .body(json!({"query": query}))
            .send()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let hits = Self::parse_hits(&body);

        hits.iter()
            .map(|(doc_id, doc)| {
                CategoryElasticSearchMapper::to_entity(doc_id, doc)
                    .map_err(|e| EsRepositoryError::Mapping(e.to_string()))
            })
            .collect()
    }

    async fn find_by_ids(
        &self,
        ids: &[CategoryId],
    ) -> Result<FindByIdsResult<Category>, Self::Error> {
        let id_strings: Vec<String> = ids.iter().map(ToString::to_string).collect();

        let mut query = json!({
            "bool": {
                "must": [
                    {"ids": {"values": id_strings}},
                    {"match": {"type": CATEGORY_DOCUMENT_TYPE}}
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
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let hits = Self::parse_hits(&body);
        let found_ids: Vec<String> = hits.iter().map(|(id, _)| id.clone()).collect();

        let exists: Result<Vec<Category>, _> = hits
            .iter()
            .map(|(doc_id, doc)| {
                CategoryElasticSearchMapper::to_entity(doc_id, doc)
                    .map_err(|e| EsRepositoryError::Mapping(e.to_string()))
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

    async fn exists_by_id(&self, ids: &[CategoryId]) -> Result<ExistsByIdResult, Self::Error> {
        let id_strings: Vec<String> = ids.iter().map(ToString::to_string).collect();

        let query = json!({
            "bool": {
                "must": [
                    {"ids": {"values": id_strings}},
                    {"match": {"type": CATEGORY_DOCUMENT_TYPE}}
                ]
            }
        });

        let response = self
            .client
            .search(SearchParts::Index(&[&self.index]))
            .body(json!({"query": query, "_source": false}))
            .send()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

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

    async fn update(&self, entity: &Category) -> Result<(), Self::Error> {
        let doc = CategoryElasticSearchMapper::to_document(entity);
        let mut params = serde_json::to_value(&doc)
            .map_err(|e| EsRepositoryError::Mapping(e.to_string()))?;

        params["category_id"] = json!(entity.category_id().to_string());
        params["is_deleted"] = json!(entity.deleted_at().is_some());

        let script = r#"
            if (ctx._source.containsKey('categories')) {
                for(item in ctx._source.categories) {
                    if (item.category_id == params.category_id) {
                        item.category_name = params.category_name;
                        item.is_active = params.is_active;
                        item.deleted_at = params.deleted_at;
                        item.is_deleted = params.is_deleted;
                    }
                }
            } else {
                ctx._source.category_name = params.category_name;
                ctx._source.category_description = params.category_description;
                ctx._source.is_active = params.is_active;
                ctx._source.created_at = params.created_at;
                ctx._source.deleted_at = params.deleted_at;
            }
        "#;

        let query = json!({
            "bool": {
                "should": [
                    {"match": {"_id": entity.category_id().to_string()}},
                    {
                        "nested": {
                            "path": "categories",
                            "query": {
                                "match": {"categories.category_id": entity.category_id().to_string()}
                            }
                        }
                    }
                ]
            }
        });

        let response = self
            .client
            .update_by_query(UpdateByQueryParts::Index(&[&self.index]))
            .body(json!({
                "query": query,
                "script": {
                    "source": script,
                    "params": params
                }
            }))
            .refresh(true)
            .send()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        if body["updated"].as_u64().unwrap_or(0) == 0 {
            return Err(NotFoundError::new(
                &entity.category_id().to_string(),
                "Category",
            )
            .into());
        }

        Ok(())
    }

    async fn has_only_one_activate_in_related(
        &self,
        id: &CategoryId,
    ) -> Result<bool, Self::Error> {
        let query = json!({
            "bool": {
                "must": [
                    {"match": {"type": "Genre"}},
                    {
                        "nested": {
                            "path": "categories",
                            "query": {
                                "bool": {
                                    "must": [
                                        {"match": {"categories.category_id": id.to_string()}},
                                        {"match": {"categories.is_active": true}}
                                    ]
                                }
                            }
                        }
                    }
                ],
                "must_not": [
                    {"match": {"categories__is_active": true}}
                ]
            }
        });

        let response = self
            .client
            .search(SearchParts::Index(&[&self.index]))
            .body(json!({"query": query, "_source": false}))
            .send()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let total = body["hits"]["total"]["value"].as_u64().unwrap_or(0);
        Ok(total >= 1)
    }

    async fn has_only_one_not_deleted_in_related(
        &self,
        id: &CategoryId,
    ) -> Result<bool, Self::Error> {
        let query = json!({
            "bool": {
                "must": [
                    {"match": {"type": "Genre"}},
                    {
                        "nested": {
                            "path": "categories",
                            "query": {
                                "bool": {
                                    "must": [
                                        {"match": {"categories.category_id": id.to_string()}},
                                        {"match": {"categories.is_deleted": false}}
                                    ]
                                }
                            }
                        }
                    }
                ],
                "must_not": [
                    {"match": {"categories__is_deleted": false}}
                ]
            }
        });

        let response = self
            .client
            .search(SearchParts::Index(&[&self.index]))
            .body(json!({"query": query, "_source": false}))
            .send()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let total = body["hits"]["total"]["value"].as_u64().unwrap_or(0);
        Ok(total >= 1)
    }

    async fn delete(&self, id: &CategoryId) -> Result<(), Self::Error> {
        let response = self
            .client
            .delete_by_query(DeleteByQueryParts::Index(&[&self.index]))
            .body(json!({"query": {"match": {"_id": id.to_string()}}}))
            .refresh(true)
            .send()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| EsRepositoryError::Elasticsearch(e.to_string()))?;

        if body["deleted"].as_u64().unwrap_or(0) == 0 {
            return Err(NotFoundError::new(&id.to_string(), "Category").into());
        }

        Ok(())
    }
}
