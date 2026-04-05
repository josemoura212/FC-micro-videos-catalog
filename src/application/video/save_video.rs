use chrono::{DateTime, Utc};

use crate::domain::cast_member::cast_member_id::CastMemberId;
use crate::domain::cast_member::cast_member_repository::ICastMemberRepository;
use crate::domain::cast_member::nested_cast_member::NestedCastMemberCreateCommand;
use crate::domain::category::category_id::CategoryId;
use crate::domain::category::category_repository::ICategoryRepository;
use crate::domain::category::nested_category::NestedCategoryCreateCommand;
use crate::domain::genre::genre_id::GenreId;
use crate::domain::genre::genre_repository::IGenreRepository;
use crate::domain::genre::nested_genre::NestedGenreCreateCommand;
use crate::domain::shared::entity::Entity;
use crate::domain::shared::errors::EntityValidationError;
use crate::domain::video::aggregate::{Video, VideoCreateCommand};
use crate::domain::video::rating::Rating;
use crate::domain::video::video_id::VideoId;
use crate::domain::video::video_repository::IVideoRepository;

#[derive(Debug, Clone)]
pub struct SaveVideoInput {
    pub video_id: String,
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
    pub categories_id: Vec<String>,
    pub genres_id: Vec<String>,
    pub cast_members_id: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SaveVideoOutput {
    pub id: String,
    pub created: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum SaveVideoError<
    VE: std::error::Error,
    CE: std::error::Error,
    GE: std::error::Error,
    ME: std::error::Error,
> {
    #[error(transparent)]
    Validation(#[from] EntityValidationError),
    #[error("invalid video id: {0}")]
    InvalidId(String),
    #[error("invalid rating: {0}")]
    InvalidRating(String),
    #[error("categories not found: {0:?}")]
    CategoriesNotFound(Vec<String>),
    #[error("genres not found: {0:?}")]
    GenresNotFound(Vec<String>),
    #[error("cast members not found: {0:?}")]
    CastMembersNotFound(Vec<String>),
    #[error(transparent)]
    VideoRepository(VE),
    #[error("category repository error: {0}")]
    CategoryRepository(CE),
    #[error("genre repository error: {0}")]
    GenreRepository(GE),
    #[error("cast member repository error: {0}")]
    CastMemberRepository(ME),
}

pub struct SaveVideoUseCase<
    VR: IVideoRepository,
    CR: ICategoryRepository,
    GR: IGenreRepository,
    MR: ICastMemberRepository,
> {
    video_repo: VR,
    category_repo: CR,
    genre_repo: GR,
    cast_member_repo: MR,
}

impl<
    VR: IVideoRepository,
    CR: ICategoryRepository,
    GR: IGenreRepository,
    MR: ICastMemberRepository,
> SaveVideoUseCase<VR, CR, GR, MR>
{
    #[must_use]
    pub const fn new(
        video_repo: VR,
        category_repo: CR,
        genre_repo: GR,
        cast_member_repo: MR,
    ) -> Self {
        Self {
            video_repo,
            category_repo,
            genre_repo,
            cast_member_repo,
        }
    }

    /// # Errors
    /// Returns error on validation failure, related entities not found, or repository error.
    pub async fn execute(
        &self,
        input: SaveVideoInput,
    ) -> Result<SaveVideoOutput, SaveVideoError<VR::Error, CR::Error, GR::Error, MR::Error>> {
        let video_id = VideoId::from(&input.video_id)
            .map_err(|e| SaveVideoError::InvalidId(e.to_string()))?;

        let existing = self
            .video_repo
            .find_by_id(&video_id)
            .await
            .map_err(SaveVideoError::VideoRepository)?;

        match existing {
            Some(video) => self.update_video(input, video).await,
            None => self.create_video(input, video_id).await,
        }
    }

    async fn create_video(
        &self,
        input: SaveVideoInput,
        video_id: VideoId,
    ) -> Result<SaveVideoOutput, SaveVideoError<VR::Error, CR::Error, GR::Error, MR::Error>> {
        let rating = Rating::from_str(&input.rating)
            .map_err(|e| SaveVideoError::InvalidRating(e.to_string()))?;

        let categories_props = self.get_categories_props(&input.categories_id).await?;
        let genres_props = self.get_genres_props(&input.genres_id).await?;
        let cast_members_props = self.get_cast_members_props(&input.cast_members_id).await?;

        let entity = Video::create(VideoCreateCommand {
            video_id,
            title: input.title,
            description: input.description,
            year_launched: input.year_launched,
            duration: input.duration,
            rating,
            is_opened: input.is_opened,
            is_published: input.is_published,
            banner_url: input.banner_url,
            thumbnail_url: input.thumbnail_url,
            thumbnail_half_url: input.thumbnail_half_url,
            trailer_url: input.trailer_url,
            video_url: input.video_url,
            categories_props,
            genres_props,
            cast_members_props,
            created_at: input.created_at,
        });

        if entity.notification().has_errors() {
            return Err(EntityValidationError::new(entity.notification().clone()).into());
        }

        self.video_repo
            .insert(&entity)
            .await
            .map_err(SaveVideoError::VideoRepository)?;

        Ok(SaveVideoOutput {
            id: entity.video_id().to_string(),
            created: true,
        })
    }

    async fn update_video(
        &self,
        input: SaveVideoInput,
        mut video: Video,
    ) -> Result<SaveVideoOutput, SaveVideoError<VR::Error, CR::Error, GR::Error, MR::Error>> {
        let rating = Rating::from_str(&input.rating)
            .map_err(|e| SaveVideoError::InvalidRating(e.to_string()))?;

        let categories_props = self.get_categories_props(&input.categories_id).await?;
        let genres_props = self.get_genres_props(&input.genres_id).await?;
        let cast_members_props = self.get_cast_members_props(&input.cast_members_id).await?;

        video.change_title(input.title);
        video.change_description(input.description);
        video.change_year_launched(input.year_launched);
        video.change_duration(input.duration);
        video.change_rating(rating);

        if input.is_opened {
            video.mark_as_opened();
        } else {
            video.mark_as_not_opened();
        }

        if input.is_published {
            video.publish();
        } else {
            video.unpublish();
        }

        video.replace_banner_url(input.banner_url);
        video.replace_thumbnail_url(input.thumbnail_url);
        video.replace_thumbnail_half_url(input.thumbnail_half_url);
        video.replace_trailer_url(input.trailer_url);
        video.replace_video_url(input.video_url);

        let nested_categories = categories_props
            .into_iter()
            .map(crate::domain::category::nested_category::NestedCategory::create)
            .collect();
        video.sync_nested_categories(nested_categories);

        let nested_genres = genres_props
            .into_iter()
            .map(crate::domain::genre::nested_genre::NestedGenre::create)
            .collect();
        video.sync_nested_genres(nested_genres);

        let nested_cast_members = cast_members_props
            .into_iter()
            .map(crate::domain::cast_member::nested_cast_member::NestedCastMember::create)
            .collect();
        video.sync_nested_cast_members(nested_cast_members);

        video.change_created_at(input.created_at);

        if video.notification().has_errors() {
            return Err(EntityValidationError::new(video.notification().clone()).into());
        }

        self.video_repo
            .update(&video)
            .await
            .map_err(SaveVideoError::VideoRepository)?;

        Ok(SaveVideoOutput {
            id: video.video_id().to_string(),
            created: false,
        })
    }

    async fn get_categories_props(
        &self,
        categories_id: &[String],
    ) -> Result<
        Vec<NestedCategoryCreateCommand>,
        SaveVideoError<VR::Error, CR::Error, GR::Error, MR::Error>,
    > {
        let category_ids: Vec<CategoryId> = categories_id
            .iter()
            .map(|id| {
                CategoryId::from(id).map_err(|e| SaveVideoError::InvalidId(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let result = self
            .category_repo
            .find_by_ids(&category_ids)
            .await
            .map_err(SaveVideoError::CategoryRepository)?;

        if !result.not_exists.is_empty() {
            let not_found_ids: Vec<String> =
                result.not_exists.iter().map(ToString::to_string).collect();
            return Err(SaveVideoError::CategoriesNotFound(not_found_ids));
        }

        Ok(result
            .exists
            .iter()
            .map(|category| NestedCategoryCreateCommand {
                category_id: category.category_id().clone(),
                name: category.name().to_string(),
                is_active: category.is_active(),
                deleted_at: category.deleted_at(),
            })
            .collect())
    }

    async fn get_genres_props(
        &self,
        genres_id: &[String],
    ) -> Result<
        Vec<NestedGenreCreateCommand>,
        SaveVideoError<VR::Error, CR::Error, GR::Error, MR::Error>,
    > {
        let genre_ids: Vec<GenreId> = genres_id
            .iter()
            .map(|id| GenreId::from(id).map_err(|e| SaveVideoError::InvalidId(e.to_string())))
            .collect::<Result<Vec<_>, _>>()?;

        let result = self
            .genre_repo
            .find_by_ids(&genre_ids)
            .await
            .map_err(SaveVideoError::GenreRepository)?;

        if !result.not_exists.is_empty() {
            let not_found_ids: Vec<String> =
                result.not_exists.iter().map(ToString::to_string).collect();
            return Err(SaveVideoError::GenresNotFound(not_found_ids));
        }

        Ok(result
            .exists
            .iter()
            .map(|genre| NestedGenreCreateCommand {
                genre_id: genre.genre_id().clone(),
                name: genre.name().to_string(),
                is_active: genre.is_active(),
                deleted_at: genre.deleted_at(),
            })
            .collect())
    }

    async fn get_cast_members_props(
        &self,
        cast_members_id: &[String],
    ) -> Result<
        Vec<NestedCastMemberCreateCommand>,
        SaveVideoError<VR::Error, CR::Error, GR::Error, MR::Error>,
    > {
        let member_ids: Vec<CastMemberId> = cast_members_id
            .iter()
            .map(|id| {
                CastMemberId::from(id).map_err(|e| SaveVideoError::InvalidId(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let result = self
            .cast_member_repo
            .find_by_ids(&member_ids)
            .await
            .map_err(SaveVideoError::CastMemberRepository)?;

        if !result.not_exists.is_empty() {
            let not_found_ids: Vec<String> =
                result.not_exists.iter().map(ToString::to_string).collect();
            return Err(SaveVideoError::CastMembersNotFound(not_found_ids));
        }

        Ok(result
            .exists
            .iter()
            .map(|member| NestedCastMemberCreateCommand {
                cast_member_id: member.cast_member_id().clone(),
                name: member.name().to_string(),
                cast_member_type: member.cast_member_type(),
                deleted_at: member.deleted_at(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cast_member::aggregate::{CastMember, CastMemberCreateCommand};
    use crate::domain::cast_member::cast_member_type::CastMemberType;
    use crate::domain::category::aggregate::{Category, CategoryCreateCommand};
    use crate::domain::genre::aggregate::{Genre, GenreCreateCommand};
    use crate::infrastructure::in_memory::cast_member_in_memory_repository::CastMemberInMemoryRepository;
    use crate::infrastructure::in_memory::category_in_memory_repository::CategoryInMemoryRepository;
    use crate::infrastructure::in_memory::genre_in_memory_repository::GenreInMemoryRepository;
    use crate::infrastructure::in_memory::video_in_memory_repository::VideoInMemoryRepository;

    async fn setup_repos() -> (
        VideoInMemoryRepository,
        CategoryInMemoryRepository,
        GenreInMemoryRepository,
        CastMemberInMemoryRepository,
        CategoryId,
        GenreId,
        CastMemberId,
    ) {
        let video_repo = VideoInMemoryRepository::new();
        let category_repo = CategoryInMemoryRepository::new();
        let genre_repo = GenreInMemoryRepository::new();
        let cast_member_repo = CastMemberInMemoryRepository::new();

        let category = Category::create(CategoryCreateCommand {
            category_id: CategoryId::new(),
            name: "Movie".to_string(),
            description: None,
            is_active: true,
            created_at: Utc::now(),
        });
        category_repo
            .insert(&category)
            .await
            .expect("insert category");

        let genre = Genre::create(GenreCreateCommand {
            genre_id: GenreId::new(),
            name: "Action".to_string(),
            categories_props: vec![],
            is_active: true,
            created_at: Utc::now(),
        });
        genre_repo.insert(&genre).await.expect("insert genre");

        let cast_member = CastMember::create(CastMemberCreateCommand {
            cast_member_id: CastMemberId::new(),
            name: "John Doe".to_string(),
            cast_member_type: CastMemberType::Actor,
            created_at: Utc::now(),
        });
        cast_member_repo
            .insert(&cast_member)
            .await
            .expect("insert cast member");

        (
            video_repo,
            category_repo,
            genre_repo,
            cast_member_repo,
            category.category_id().clone(),
            genre.genre_id().clone(),
            cast_member.cast_member_id().clone(),
        )
    }

    #[tokio::test]
    async fn should_create_video() {
        let (video_repo, category_repo, genre_repo, cast_member_repo, cat_id, genre_id, member_id) =
            setup_repos().await;

        let use_case =
            SaveVideoUseCase::new(video_repo, category_repo, genre_repo, cast_member_repo);
        let video_id = VideoId::new();

        let output = use_case
            .execute(SaveVideoInput {
                video_id: video_id.to_string(),
                title: "Test Video".to_string(),
                description: "A description".to_string(),
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
                categories_id: vec![cat_id.to_string()],
                genres_id: vec![genre_id.to_string()],
                cast_members_id: vec![member_id.to_string()],
                created_at: Utc::now(),
            })
            .await
            .expect("should create");

        assert_eq!(output.id, video_id.to_string());
        assert!(output.created);
    }

    #[tokio::test]
    async fn should_update_video() {
        let (video_repo, category_repo, genre_repo, cast_member_repo, cat_id, genre_id, member_id) =
            setup_repos().await;

        let video = Video::create(VideoCreateCommand {
            video_id: VideoId::new(),
            title: "Original".to_string(),
            description: "Original desc".to_string(),
            year_launched: 2024,
            duration: 90,
            rating: Rating::R10,
            is_opened: false,
            is_published: false,
            banner_url: None,
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
                cast_member_id: member_id.clone(),
                name: "John Doe".to_string(),
                cast_member_type: CastMemberType::Actor,
                deleted_at: None,
            }],
            created_at: Utc::now(),
        });
        video_repo.insert(&video).await.expect("insert video");

        let use_case =
            SaveVideoUseCase::new(video_repo, category_repo, genre_repo, cast_member_repo);
        let output = use_case
            .execute(SaveVideoInput {
                video_id: video.video_id().to_string(),
                title: "Updated".to_string(),
                description: "Updated desc".to_string(),
                year_launched: 2025,
                duration: 150,
                rating: "14".to_string(),
                is_opened: true,
                is_published: true,
                banner_url: Some("http://banner.jpg".to_string()),
                thumbnail_url: None,
                thumbnail_half_url: None,
                trailer_url: "http://trailer2.mp4".to_string(),
                video_url: "http://video2.mp4".to_string(),
                categories_id: vec![cat_id.to_string()],
                genres_id: vec![genre_id.to_string()],
                cast_members_id: vec![member_id.to_string()],
                created_at: Utc::now(),
            })
            .await
            .expect("should update");

        assert_eq!(output.id, video.video_id().to_string());
        assert!(!output.created);
    }

    #[tokio::test]
    async fn should_fail_when_categories_not_found() {
        let (video_repo, category_repo, genre_repo, cast_member_repo, _, genre_id, member_id) =
            setup_repos().await;

        let use_case =
            SaveVideoUseCase::new(video_repo, category_repo, genre_repo, cast_member_repo);

        let result = use_case
            .execute(SaveVideoInput {
                video_id: VideoId::new().to_string(),
                title: "Test".to_string(),
                description: "Desc".to_string(),
                year_launched: 2024,
                duration: 90,
                rating: "L".to_string(),
                is_opened: false,
                is_published: false,
                banner_url: None,
                thumbnail_url: None,
                thumbnail_half_url: None,
                trailer_url: "http://trailer.mp4".to_string(),
                video_url: "http://video.mp4".to_string(),
                categories_id: vec![CategoryId::new().to_string()],
                genres_id: vec![genre_id.to_string()],
                cast_members_id: vec![member_id.to_string()],
                created_at: Utc::now(),
            })
            .await;

        assert!(matches!(result, Err(SaveVideoError::CategoriesNotFound(_))));
    }

    #[tokio::test]
    async fn should_fail_when_genres_not_found() {
        let (video_repo, category_repo, genre_repo, cast_member_repo, cat_id, _, member_id) =
            setup_repos().await;

        let use_case =
            SaveVideoUseCase::new(video_repo, category_repo, genre_repo, cast_member_repo);

        let result = use_case
            .execute(SaveVideoInput {
                video_id: VideoId::new().to_string(),
                title: "Test".to_string(),
                description: "Desc".to_string(),
                year_launched: 2024,
                duration: 90,
                rating: "L".to_string(),
                is_opened: false,
                is_published: false,
                banner_url: None,
                thumbnail_url: None,
                thumbnail_half_url: None,
                trailer_url: "http://trailer.mp4".to_string(),
                video_url: "http://video.mp4".to_string(),
                categories_id: vec![cat_id.to_string()],
                genres_id: vec![GenreId::new().to_string()],
                cast_members_id: vec![member_id.to_string()],
                created_at: Utc::now(),
            })
            .await;

        assert!(matches!(result, Err(SaveVideoError::GenresNotFound(_))));
    }

    #[tokio::test]
    async fn should_fail_when_cast_members_not_found() {
        let (video_repo, category_repo, genre_repo, cast_member_repo, cat_id, genre_id, _) =
            setup_repos().await;

        let use_case =
            SaveVideoUseCase::new(video_repo, category_repo, genre_repo, cast_member_repo);

        let result = use_case
            .execute(SaveVideoInput {
                video_id: VideoId::new().to_string(),
                title: "Test".to_string(),
                description: "Desc".to_string(),
                year_launched: 2024,
                duration: 90,
                rating: "L".to_string(),
                is_opened: false,
                is_published: false,
                banner_url: None,
                thumbnail_url: None,
                thumbnail_half_url: None,
                trailer_url: "http://trailer.mp4".to_string(),
                video_url: "http://video.mp4".to_string(),
                categories_id: vec![cat_id.to_string()],
                genres_id: vec![genre_id.to_string()],
                cast_members_id: vec![CastMemberId::new().to_string()],
                created_at: Utc::now(),
            })
            .await;

        assert!(matches!(
            result,
            Err(SaveVideoError::CastMembersNotFound(_))
        ));
    }

    #[tokio::test]
    async fn should_fail_with_invalid_rating() {
        let (video_repo, category_repo, genre_repo, cast_member_repo, cat_id, genre_id, member_id) =
            setup_repos().await;

        let use_case =
            SaveVideoUseCase::new(video_repo, category_repo, genre_repo, cast_member_repo);

        let result = use_case
            .execute(SaveVideoInput {
                video_id: VideoId::new().to_string(),
                title: "Test".to_string(),
                description: "Desc".to_string(),
                year_launched: 2024,
                duration: 90,
                rating: "PG".to_string(),
                is_opened: false,
                is_published: false,
                banner_url: None,
                thumbnail_url: None,
                thumbnail_half_url: None,
                trailer_url: "http://trailer.mp4".to_string(),
                video_url: "http://video.mp4".to_string(),
                categories_id: vec![cat_id.to_string()],
                genres_id: vec![genre_id.to_string()],
                cast_members_id: vec![member_id.to_string()],
                created_at: Utc::now(),
            })
            .await;

        assert!(matches!(result, Err(SaveVideoError::InvalidRating(_))));
    }
}
