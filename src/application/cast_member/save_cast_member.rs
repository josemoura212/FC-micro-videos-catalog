use chrono::{DateTime, Utc};

use crate::domain::cast_member::aggregate::{CastMember, CastMemberCreateCommand};
use crate::domain::cast_member::cast_member_id::CastMemberId;
use crate::domain::cast_member::cast_member_repository::ICastMemberRepository;
use crate::domain::cast_member::cast_member_type::CastMemberType;
use crate::domain::shared::entity::Entity;
use crate::domain::shared::errors::EntityValidationError;

#[derive(Debug, Clone)]
pub struct SaveCastMemberInput {
    pub cast_member_id: String,
    pub name: String,
    pub cast_member_type: u8,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SaveCastMemberOutput {
    pub id: String,
    pub created: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum SaveCastMemberError<E: std::error::Error> {
    #[error(transparent)]
    Validation(#[from] EntityValidationError),
    #[error("invalid cast member id: {0}")]
    InvalidId(String),
    #[error("invalid cast member type: {0}")]
    InvalidType(String),
    #[error(transparent)]
    Repository(E),
}

pub struct SaveCastMemberUseCase<R: ICastMemberRepository> {
    repo: R,
}

impl<R: ICastMemberRepository> SaveCastMemberUseCase<R> {
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// # Errors
    /// Returns error on validation failure or repository error.
    pub async fn execute(
        &self,
        input: SaveCastMemberInput,
    ) -> Result<SaveCastMemberOutput, SaveCastMemberError<R::Error>> {
        let cast_member_id = CastMemberId::from(&input.cast_member_id)
            .map_err(|e| SaveCastMemberError::InvalidId(e.to_string()))?;

        let cast_member_type = CastMemberType::from_u8(input.cast_member_type)
            .map_err(|e| SaveCastMemberError::InvalidType(e.to_string()))?;

        let existing = self
            .repo
            .find_by_id(&cast_member_id)
            .await
            .map_err(SaveCastMemberError::Repository)?;

        match existing {
            Some(entity) => self.update_cast_member(input, entity, cast_member_type).await,
            None => {
                self.create_cast_member(input, cast_member_id, cast_member_type)
                    .await
            }
        }
    }

    async fn create_cast_member(
        &self,
        input: SaveCastMemberInput,
        cast_member_id: CastMemberId,
        cast_member_type: CastMemberType,
    ) -> Result<SaveCastMemberOutput, SaveCastMemberError<R::Error>> {
        let entity = CastMember::create(CastMemberCreateCommand {
            cast_member_id,
            name: input.name,
            cast_member_type,
            created_at: input.created_at,
        });

        if entity.notification().has_errors() {
            return Err(EntityValidationError::new(entity.notification().clone()).into());
        }

        self.repo
            .insert(&entity)
            .await
            .map_err(SaveCastMemberError::Repository)?;

        Ok(SaveCastMemberOutput {
            id: entity.cast_member_id().to_string(),
            created: true,
        })
    }

    async fn update_cast_member(
        &self,
        input: SaveCastMemberInput,
        mut cast_member: CastMember,
        cast_member_type: CastMemberType,
    ) -> Result<SaveCastMemberOutput, SaveCastMemberError<R::Error>> {
        cast_member.change_name(input.name);
        cast_member.change_type(cast_member_type);
        cast_member.change_created_at(input.created_at);

        if cast_member.notification().has_errors() {
            return Err(EntityValidationError::new(cast_member.notification().clone()).into());
        }

        self.repo
            .update(&cast_member)
            .await
            .map_err(SaveCastMemberError::Repository)?;

        Ok(SaveCastMemberOutput {
            id: cast_member.cast_member_id().to_string(),
            created: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::in_memory::cast_member_in_memory_repository::CastMemberInMemoryRepository;

    #[tokio::test]
    async fn should_create_cast_member() {
        let repo = CastMemberInMemoryRepository::new();
        let use_case = SaveCastMemberUseCase::new(repo);
        let cast_member_id = CastMemberId::new();

        let output = use_case
            .execute(SaveCastMemberInput {
                cast_member_id: cast_member_id.to_string(),
                name: "John Doe".to_string(),
                cast_member_type: 2,
                created_at: Utc::now(),
            })
            .await
            .expect("should create");

        assert_eq!(output.id, cast_member_id.to_string());
        assert!(output.created);
    }

    #[tokio::test]
    async fn should_update_cast_member() {
        let repo = CastMemberInMemoryRepository::new();
        let cast_member = CastMember::create(CastMemberCreateCommand {
            cast_member_id: CastMemberId::new(),
            name: "John Doe".to_string(),
            cast_member_type: CastMemberType::Actor,
            created_at: Utc::now(),
        });
        repo.insert(&cast_member).await.expect("insert");

        let use_case = SaveCastMemberUseCase::new(repo);
        let output = use_case
            .execute(SaveCastMemberInput {
                cast_member_id: cast_member.cast_member_id().to_string(),
                name: "Jane Director".to_string(),
                cast_member_type: 1,
                created_at: Utc::now(),
            })
            .await
            .expect("should update");

        assert_eq!(output.id, cast_member.cast_member_id().to_string());
        assert!(!output.created);
    }

    #[tokio::test]
    async fn should_fail_with_invalid_name() {
        let repo = CastMemberInMemoryRepository::new();
        let use_case = SaveCastMemberUseCase::new(repo);

        let result = use_case
            .execute(SaveCastMemberInput {
                cast_member_id: CastMemberId::new().to_string(),
                name: "a".repeat(256),
                cast_member_type: 2,
                created_at: Utc::now(),
            })
            .await;

        assert!(result.is_err());
    }
}
