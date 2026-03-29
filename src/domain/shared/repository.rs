use std::collections::HashMap;

use async_trait::async_trait;

use super::entity::AggregateRoot;
use super::value_object::UuidVo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    #[must_use] 
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchParams<Filter = String> {
    pub page: usize,
    pub per_page: usize,
    pub sort: Option<String>,
    pub sort_dir: Option<SortDirection>,
    pub filter: Option<Filter>,
}

impl<Filter> Default for SearchParams<Filter> {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 15,
            sort: None,
            sort_dir: None,
            filter: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult<E> {
    pub items: Vec<E>,
    pub total: usize,
    pub current_page: usize,
    pub per_page: usize,
}

impl<E> SearchResult<E> {
    #[must_use] 
    pub const fn last_page(&self) -> usize {
        if self.total == 0 {
            return 1;
        }
        self.total.div_ceil(self.per_page)
    }
}

pub struct FindByIdsResult<E> {
    pub exists: Vec<E>,
    pub not_exists: Vec<UuidVo>,
}

pub struct ExistsByIdResult {
    pub exists: Vec<UuidVo>,
    pub not_exists: Vec<UuidVo>,
}

#[derive(Debug, Clone)]
pub struct SortOrder {
    pub field: String,
    pub direction: SortDirection,
}

#[async_trait]
pub trait Repository<E: AggregateRoot>: Send + Sync {
    type Error: std::error::Error + Send + Sync;

    fn sortable_fields(&self) -> &[&str];

    async fn insert(&self, entity: &E) -> Result<(), Self::Error>;
    async fn bulk_insert(&self, entities: &[E]) -> Result<(), Self::Error>;
    async fn find_by_id(&self, id: &UuidVo) -> Result<Option<E>, Self::Error>;
    async fn find_one_by(
        &self,
        filter: &HashMap<String, serde_json::Value>,
    ) -> Result<Option<E>, Self::Error>;
    async fn find_by(
        &self,
        filter: &HashMap<String, serde_json::Value>,
        order: Option<&SortOrder>,
    ) -> Result<Vec<E>, Self::Error>;
    async fn find_all(&self) -> Result<Vec<E>, Self::Error>;
    async fn find_by_ids(
        &self,
        ids: &[UuidVo],
    ) -> Result<FindByIdsResult<E>, Self::Error>;
    async fn exists_by_id(
        &self,
        ids: &[UuidVo],
    ) -> Result<ExistsByIdResult, Self::Error>;
    async fn update(&self, entity: &E) -> Result<(), Self::Error>;
    async fn delete(&self, id: &UuidVo) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_params_defaults() {
        let params = SearchParams::<String>::default();
        assert_eq!(params.page, 1);
        assert_eq!(params.per_page, 15);
        assert!(params.sort.is_none());
        assert!(params.filter.is_none());
    }

    #[test]
    fn search_result_last_page() {
        let result: SearchResult<()> = SearchResult {
            items: vec![],
            total: 97,
            current_page: 1,
            per_page: 15,
        };
        assert_eq!(result.last_page(), 7);
    }

    #[test]
    fn search_result_last_page_empty() {
        let result: SearchResult<()> = SearchResult {
            items: vec![],
            total: 0,
            current_page: 1,
            per_page: 15,
        };
        assert_eq!(result.last_page(), 1);
    }
}
