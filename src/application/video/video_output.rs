use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domain::cast_member::cast_member_type::CastMemberType;
use crate::domain::video::aggregate::Video;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NestedCategoryOutput {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NestedGenreOutput {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NestedCastMemberOutput {
    pub id: String,
    pub name: String,
    pub cast_member_type: CastMemberType,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VideoOutput {
    pub id: String,
    pub title: String,
    pub description: String,
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
    pub categories: Vec<NestedCategoryOutput>,
    pub genres: Vec<NestedGenreOutput>,
    pub cast_members: Vec<NestedCastMemberOutput>,
    pub created_at: DateTime<Utc>,
}

pub struct VideoOutputMapper;

impl VideoOutputMapper {
    #[must_use]
    pub fn to_output(entity: &Video) -> VideoOutput {
        let mut categories: Vec<NestedCategoryOutput> = entity
            .categories()
            .values()
            .map(|nested| NestedCategoryOutput {
                id: nested.category_id().to_string(),
                name: nested.name().to_string(),
                is_active: nested.is_active(),
                deleted_at: nested.deleted_at(),
            })
            .collect();
        categories.sort_by(|a, b| a.name.cmp(&b.name));

        let mut genres: Vec<NestedGenreOutput> = entity
            .genres()
            .values()
            .map(|nested| NestedGenreOutput {
                id: nested.genre_id().to_string(),
                name: nested.name().to_string(),
                is_active: nested.is_active(),
                deleted_at: nested.deleted_at(),
            })
            .collect();
        genres.sort_by(|a, b| a.name.cmp(&b.name));

        let mut cast_members: Vec<NestedCastMemberOutput> = entity
            .cast_members()
            .values()
            .map(|nested| NestedCastMemberOutput {
                id: nested.cast_member_id().to_string(),
                name: nested.name().to_string(),
                cast_member_type: nested.cast_member_type(),
                deleted_at: nested.deleted_at(),
            })
            .collect();
        cast_members.sort_by(|a, b| a.name.cmp(&b.name));

        VideoOutput {
            id: entity.video_id().to_string(),
            title: entity.title().to_string(),
            description: entity.description().to_string(),
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
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::domain::cast_member::cast_member_id::CastMemberId;
    use crate::domain::cast_member::nested_cast_member::NestedCastMemberCreateCommand;
    use crate::domain::category::category_id::CategoryId;
    use crate::domain::category::nested_category::NestedCategoryCreateCommand;
    use crate::domain::genre::genre_id::GenreId;
    use crate::domain::genre::nested_genre::NestedGenreCreateCommand;
    use crate::domain::video::aggregate::VideoCreateCommand;
    use crate::domain::video::rating::Rating;
    use crate::domain::video::video_id::VideoId;

    use super::*;

    #[test]
    fn should_convert_video_to_output() {
        let cat_id = CategoryId::new();
        let genre_id = GenreId::new();
        let cast_member_id = CastMemberId::new();

        let video = Video::create(VideoCreateCommand {
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
                category_id: cat_id.clone(),
                name: "Movie".to_string(),
                is_active: true,
                deleted_at: None,
            }],
            genres_props: vec![NestedGenreCreateCommand {
                genre_id: genre_id.clone(),
                name: "Action".to_string(),
                is_active: true,
                deleted_at: None,
            }],
            cast_members_props: vec![NestedCastMemberCreateCommand {
                cast_member_id: cast_member_id.clone(),
                name: "John Doe".to_string(),
                cast_member_type: CastMemberType::Actor,
                deleted_at: None,
            }],
            created_at: Utc::now(),
        });

        let output = VideoOutputMapper::to_output(&video);
        assert_eq!(output.id, video.video_id().to_string());
        assert_eq!(output.title, "Test Video");
        assert_eq!(output.description, "A test description");
        assert_eq!(output.year_launched, 2024);
        assert_eq!(output.duration, 120);
        assert_eq!(output.rating, "12");
        assert!(!output.is_opened);
        assert!(output.is_published);
        assert_eq!(output.banner_url, Some("http://banner.jpg".to_string()));
        assert!(output.thumbnail_url.is_none());
        assert_eq!(output.trailer_url, "http://trailer.mp4");
        assert_eq!(output.video_url, "http://video.mp4");
        assert_eq!(output.categories.len(), 1);
        assert_eq!(output.categories[0].id, cat_id.to_string());
        assert_eq!(output.categories[0].name, "Movie");
        assert_eq!(output.genres.len(), 1);
        assert_eq!(output.genres[0].id, genre_id.to_string());
        assert_eq!(output.genres[0].name, "Action");
        assert_eq!(output.cast_members.len(), 1);
        assert_eq!(output.cast_members[0].id, cast_member_id.to_string());
        assert_eq!(output.cast_members[0].name, "John Doe");
        assert_eq!(output.cast_members[0].cast_member_type, CastMemberType::Actor);
    }

    #[test]
    fn should_convert_video_without_relations() {
        let video = Video::create(VideoCreateCommand {
            video_id: VideoId::new(),
            title: "Minimal Video".to_string(),
            description: "Minimal".to_string(),
            year_launched: 2020,
            duration: 60,
            rating: Rating::RL,
            is_opened: true,
            is_published: false,
            banner_url: None,
            thumbnail_url: None,
            thumbnail_half_url: None,
            trailer_url: "http://trailer.mp4".to_string(),
            video_url: "http://video.mp4".to_string(),
            categories_props: vec![],
            genres_props: vec![],
            cast_members_props: vec![],
            created_at: Utc::now(),
        });

        let output = VideoOutputMapper::to_output(&video);
        assert_eq!(output.title, "Minimal Video");
        assert_eq!(output.rating, "L");
        assert!(output.is_opened);
        assert!(!output.is_published);
        assert!(output.categories.is_empty());
        assert!(output.genres.is_empty());
        assert!(output.cast_members.is_empty());
    }
}
