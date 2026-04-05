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
                "genre_name": { "type": "keyword" },
                "categories": {
                    "type": "nested",
                    "properties": {
                        "category_id": { "type": "keyword" },
                        "category_name": { "type": "keyword" },
                        "is_active": { "type": "boolean" },
                        "deleted_at": { "type": "date" },
                        "is_deleted": { "type": "boolean" }
                    }
                },
                "video_title": {
                    "type": "text",
                    "fields": {
                        "keyword": { "type": "keyword" }
                    }
                },
                "video_description": { "type": "text" },
                "year_launched": { "type": "integer" },
                "duration": { "type": "integer" },
                "rating": { "type": "keyword" },
                "is_opened": { "type": "boolean" },
                "is_published": { "type": "boolean" },
                "banner_url": { "type": "keyword" },
                "thumbnail_url": { "type": "keyword" },
                "thumbnail_half_url": { "type": "keyword" },
                "trailer_url": { "type": "keyword" },
                "video_url": { "type": "keyword" },
                "genres": {
                    "type": "nested",
                    "properties": {
                        "genre_id": { "type": "keyword" },
                        "genre_name": { "type": "keyword" },
                        "is_active": { "type": "boolean" },
                        "deleted_at": { "type": "date" },
                        "is_deleted": { "type": "boolean" }
                    }
                },
                "cast_members": {
                    "type": "nested",
                    "properties": {
                        "cast_member_id": { "type": "keyword" },
                        "cast_member_name": { "type": "keyword" },
                        "cast_member_type": { "type": "integer" },
                        "deleted_at": { "type": "date" },
                        "is_deleted": { "type": "boolean" }
                    }
                },
                "is_active": { "type": "boolean" },
                "created_at": { "type": "date" },
                "deleted_at": { "type": "date" }
            }
        }
    })
}
