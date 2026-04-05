use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::cast_member::cast_member_id::CastMemberId;
use crate::domain::cast_member::cast_member_type::CastMemberType;
use crate::domain::cast_member::nested_cast_member::NestedCastMember;
use crate::domain::category::category_id::CategoryId;
use crate::domain::category::nested_category::NestedCategory;
use crate::domain::genre::genre_id::GenreId;
use crate::domain::genre::nested_genre::NestedGenre;
use crate::domain::shared::errors::LoadEntityError;
use crate::domain::video::aggregate::Video;
use crate::domain::video::rating::Rating;
use crate::domain::video::video_id::VideoId;

pub const VIDEO_DOCUMENT_TYPE: &str = "Video";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedCategoryDocument {
    pub category_id: String,
    pub category_name: String,
    pub is_active: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedGenreDocument {
    pub genre_id: String,
    pub genre_name: String,
    pub is_active: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedCastMemberDocument {
    pub cast_member_id: String,
    pub cast_member_name: String,
    pub cast_member_type: u8,
    pub deleted_at: Option<DateTime<Utc>>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDocument {
    pub video_title: String,
    pub video_description: String,
    pub year_launched: i32,
    pub duration: i32,
    pub rating: String,
    pub is_opened: bool,
    pub is_published: bool,
    pub banner_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub thumbnail_half_url: Option<String>,
    pub trailer_url: String,
    pub video_url: String,
    pub categories: Vec<NestedCategoryDocument>,
    pub genres: Vec<NestedGenreDocument>,
    pub cast_members: Vec<NestedCastMemberDocument>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(rename = "type")]
    pub doc_type: String,
}

pub struct VideoElasticSearchMapper;

impl VideoElasticSearchMapper {
    /// # Errors
    /// Returns `LoadEntityError` if the document type is invalid or the entity fails validation.
    pub fn to_entity(id: &str, document: &VideoDocument) -> Result<Video, LoadEntityError> {
        if document.doc_type != VIDEO_DOCUMENT_TYPE {
            return Err(LoadEntityError {
                errors: vec!["Invalid document type".to_string()],
            });
        }

        let video_id = VideoId::from(id).map_err(|e| LoadEntityError {
            errors: vec![e.to_string()],
        })?;

        let rating = Rating::from_str(&document.rating).map_err(|e| LoadEntityError {
            errors: vec![e.to_string()],
        })?;

        let mut categories = HashMap::new();
        for cat_doc in &document.categories {
            let category_id =
                CategoryId::from(&cat_doc.category_id).map_err(|e| LoadEntityError {
                    errors: vec![e.to_string()],
                })?;

            let nested = NestedCategory::new(
                category_id,
                cat_doc.category_name.clone(),
                cat_doc.is_active,
                cat_doc.deleted_at,
            );

            categories.insert(cat_doc.category_id.clone(), nested);
        }

        let mut genres = HashMap::new();
        for genre_doc in &document.genres {
            let genre_id =
                GenreId::from(&genre_doc.genre_id).map_err(|e| LoadEntityError {
                    errors: vec![e.to_string()],
                })?;

            let nested = NestedGenre::new(
                genre_id,
                genre_doc.genre_name.clone(),
                genre_doc.is_active,
                genre_doc.deleted_at,
            );

            genres.insert(genre_doc.genre_id.clone(), nested);
        }

        let mut cast_members = HashMap::new();
        for member_doc in &document.cast_members {
            let cast_member_id =
                CastMemberId::from(&member_doc.cast_member_id).map_err(|e| LoadEntityError {
                    errors: vec![e.to_string()],
                })?;

            let cast_member_type = CastMemberType::from_u8(member_doc.cast_member_type)
                .map_err(|e| LoadEntityError {
                    errors: vec![e.to_string()],
                })?;

            let nested = NestedCastMember::new(
                cast_member_id,
                member_doc.cast_member_name.clone(),
                cast_member_type,
                member_doc.deleted_at,
            );

            cast_members.insert(member_doc.cast_member_id.clone(), nested);
        }

        let video = Video::new(
            video_id,
            document.video_title.clone(),
            document.video_description.clone(),
            document.year_launched,
            document.duration,
            rating,
            document.is_opened,
            document.is_published,
            document.banner_url.clone(),
            document.thumbnail_url.clone(),
            document.thumbnail_half_url.clone(),
            document.trailer_url.clone(),
            document.video_url.clone(),
            categories,
            genres,
            cast_members,
            document.created_at,
            document.deleted_at,
        );

        Ok(video)
    }

    #[must_use]
    pub fn to_document(entity: &Video) -> VideoDocument {
        let categories: Vec<NestedCategoryDocument> = entity
            .categories()
            .values()
            .map(|nested| NestedCategoryDocument {
                category_id: nested.category_id().to_string(),
                category_name: nested.name().to_string(),
                is_active: nested.is_active(),
                deleted_at: nested.deleted_at(),
                is_deleted: nested.deleted_at().is_some(),
            })
            .collect();

        let genres: Vec<NestedGenreDocument> = entity
            .genres()
            .values()
            .map(|nested| NestedGenreDocument {
                genre_id: nested.genre_id().to_string(),
                genre_name: nested.name().to_string(),
                is_active: nested.is_active(),
                deleted_at: nested.deleted_at(),
                is_deleted: nested.deleted_at().is_some(),
            })
            .collect();

        let cast_members: Vec<NestedCastMemberDocument> = entity
            .cast_members()
            .values()
            .map(|nested| NestedCastMemberDocument {
                cast_member_id: nested.cast_member_id().to_string(),
                cast_member_name: nested.name().to_string(),
                cast_member_type: nested.cast_member_type() as u8,
                deleted_at: nested.deleted_at(),
                is_deleted: nested.deleted_at().is_some(),
            })
            .collect();

        VideoDocument {
            video_title: entity.title().to_string(),
            video_description: entity.description().to_string(),
            year_launched: entity.year_launched(),
            duration: entity.duration(),
            rating: entity.rating().to_string(),
            is_opened: entity.is_opened(),
            is_published: entity.is_published(),
            banner_url: entity.banner_url().map(String::from),
            thumbnail_url: entity.thumbnail_url().map(String::from),
            thumbnail_half_url: entity.thumbnail_half_url().map(String::from),
            trailer_url: entity.trailer_url().to_string(),
            video_url: entity.video_url().to_string(),
            categories,
            genres,
            cast_members,
            created_at: entity.created_at(),
            deleted_at: entity.deleted_at(),
            doc_type: VIDEO_DOCUMENT_TYPE.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::cast_member::cast_member_id::CastMemberId;
    use crate::domain::cast_member::nested_cast_member::NestedCastMemberCreateCommand;
    use crate::domain::category::category_id::CategoryId;
    use crate::domain::category::nested_category::NestedCategoryCreateCommand;
    use crate::domain::genre::genre_id::GenreId;
    use crate::domain::genre::nested_genre::NestedGenreCreateCommand;
    use crate::domain::video::aggregate::VideoCreateCommand;

    fn make_video() -> Video {
        Video::create(VideoCreateCommand {
            video_id: VideoId::new(),
            title: "Test Video".to_string(),
            description: "A test description".to_string(),
            year_launched: 2024,
            duration: 120,
            rating: Rating::R12,
            is_opened: false,
            is_published: true,
            banner_url: Some("http://banner.jpg".to_string()),
            thumbnail_url: None,
            thumbnail_half_url: None,
            trailer_url: "http://trailer.mp4".to_string(),
            video_url: "http://video.mp4".to_string(),
            categories_props: vec![NestedCategoryCreateCommand {
                category_id: CategoryId::new(),
                name: "Movie".to_string(),
                is_active: true,
                deleted_at: None,
            }],
            genres_props: vec![NestedGenreCreateCommand {
                genre_id: GenreId::new(),
                name: "Action".to_string(),
                is_active: true,
                deleted_at: None,
            }],
            cast_members_props: vec![NestedCastMemberCreateCommand {
                cast_member_id: CastMemberId::new(),
                name: "John Doe".to_string(),
                cast_member_type: CastMemberType::Actor,
                deleted_at: None,
            }],
            created_at: Utc::now(),
        })
    }

    #[test]
    fn should_convert_entity_to_document() {
        let video = make_video();
        let doc = VideoElasticSearchMapper::to_document(&video);

        assert_eq!(doc.video_title, "Test Video");
        assert_eq!(doc.video_description, "A test description");
        assert_eq!(doc.year_launched, 2024);
        assert_eq!(doc.duration, 120);
        assert_eq!(doc.rating, "12");
        assert!(!doc.is_opened);
        assert!(doc.is_published);
        assert_eq!(doc.banner_url, Some("http://banner.jpg".to_string()));
        assert!(doc.thumbnail_url.is_none());
        assert_eq!(doc.trailer_url, "http://trailer.mp4");
        assert_eq!(doc.video_url, "http://video.mp4");
        assert!(doc.deleted_at.is_none());
        assert_eq!(doc.doc_type, "Video");
        assert_eq!(doc.categories.len(), 1);
        assert_eq!(doc.categories[0].category_name, "Movie");
        assert!(doc.categories[0].is_active);
        assert!(!doc.categories[0].is_deleted);
        assert_eq!(doc.genres.len(), 1);
        assert_eq!(doc.genres[0].genre_name, "Action");
        assert_eq!(doc.cast_members.len(), 1);
        assert_eq!(doc.cast_members[0].cast_member_name, "John Doe");
        assert_eq!(doc.cast_members[0].cast_member_type, 2);
    }

    #[test]
    fn should_convert_document_to_entity() {
        let category_id = CategoryId::new();
        let genre_id = GenreId::new();
        let cast_member_id = CastMemberId::new();

        let doc = VideoDocument {
            video_title: "Test Video".to_string(),
            video_description: "A test description".to_string(),
            year_launched: 2024,
            duration: 120,
            rating: "12".to_string(),
            is_opened: false,
            is_published: true,
            banner_url: Some("http://banner.jpg".to_string()),
            thumbnail_url: None,
            thumbnail_half_url: None,
            trailer_url: "http://trailer.mp4".to_string(),
            video_url: "http://video.mp4".to_string(),
            categories: vec![NestedCategoryDocument {
                category_id: category_id.to_string(),
                category_name: "Movie".to_string(),
                is_active: true,
                deleted_at: None,
                is_deleted: false,
            }],
            genres: vec![NestedGenreDocument {
                genre_id: genre_id.to_string(),
                genre_name: "Action".to_string(),
                is_active: true,
                deleted_at: None,
                is_deleted: false,
            }],
            cast_members: vec![NestedCastMemberDocument {
                cast_member_id: cast_member_id.to_string(),
                cast_member_name: "John Doe".to_string(),
                cast_member_type: 2,
                deleted_at: None,
                is_deleted: false,
            }],
            created_at: Utc::now(),
            deleted_at: None,
            doc_type: VIDEO_DOCUMENT_TYPE.to_string(),
        };

        let video_id = VideoId::new();
        let entity = VideoElasticSearchMapper::to_entity(&video_id.to_string(), &doc)
            .expect("should map");

        assert_eq!(entity.title(), "Test Video");
        assert_eq!(entity.description(), "A test description");
        assert_eq!(entity.year_launched(), 2024);
        assert_eq!(entity.duration(), 120);
        assert_eq!(entity.rating(), &Rating::R12);
        assert!(!entity.is_opened());
        assert!(entity.is_published());
        assert_eq!(entity.categories().len(), 1);
        assert_eq!(entity.genres().len(), 1);
        assert_eq!(entity.cast_members().len(), 1);
    }

    #[test]
    fn should_fail_with_invalid_type() {
        let doc = VideoDocument {
            video_title: "Test".to_string(),
            video_description: "Desc".to_string(),
            year_launched: 2024,
            duration: 120,
            rating: "12".to_string(),
            is_opened: false,
            is_published: false,
            banner_url: None,
            thumbnail_url: None,
            thumbnail_half_url: None,
            trailer_url: "http://trailer.mp4".to_string(),
            video_url: "http://video.mp4".to_string(),
            categories: vec![],
            genres: vec![],
            cast_members: vec![],
            created_at: Utc::now(),
            deleted_at: None,
            doc_type: "Invalid".to_string(),
        };

        let result = VideoElasticSearchMapper::to_entity("some-id", &doc);
        assert!(result.is_err());
    }

    #[test]
    fn should_roundtrip_entity_through_document() {
        let video = make_video();
        let doc = VideoElasticSearchMapper::to_document(&video);
        let restored =
            VideoElasticSearchMapper::to_entity(&video.video_id().to_string(), &doc)
                .expect("should roundtrip");

        assert_eq!(restored.title(), video.title());
        assert_eq!(restored.description(), video.description());
        assert_eq!(restored.year_launched(), video.year_launched());
        assert_eq!(restored.duration(), video.duration());
        assert_eq!(restored.rating(), video.rating());
        assert_eq!(restored.is_opened(), video.is_opened());
        assert_eq!(restored.is_published(), video.is_published());
        assert_eq!(restored.categories().len(), video.categories().len());
        assert_eq!(restored.genres().len(), video.genres().len());
        assert_eq!(restored.cast_members().len(), video.cast_members().len());
    }
}
