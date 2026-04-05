use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum CdcOperation {
    #[serde(rename = "r")]
    Read,
    #[serde(rename = "c")]
    Create,
    #[serde(rename = "u")]
    Update,
    #[serde(rename = "d")]
    Delete,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CdcPayload<T> {
    pub op: CdcOperation,
    pub before: Option<T>,
    pub after: Option<T>,
}
