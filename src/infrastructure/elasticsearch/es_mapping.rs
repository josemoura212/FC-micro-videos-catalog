use serde_json::{json, Value};

pub const INDEX_MAPPING: &str = "categories";

#[must_use]
pub fn es_mapping() -> Value {
    json!({
        "mappings": {
            "properties": {
                "type": { "type": "keyword" },
                "category_name": { "type": "keyword" },
                "category_description": { "type": "text" },
                "cast_member_name": { "type": "keyword" },
                "cast_member_type": { "type": "integer" },
                "is_active": { "type": "boolean" },
                "created_at": { "type": "date" },
                "deleted_at": { "type": "date" }
            }
        }
    })
}
