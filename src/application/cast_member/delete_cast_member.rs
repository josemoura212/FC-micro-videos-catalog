use crate::domain::cast_member::cast_member_id::CastMemberId;
use crate::domain::cast_member::cast_member_repository::ICastMemberRepository;
use crate::domain::shared::errors::NotFoundError;

#[derive(Debug, Clone)]
pub struct DeleteCastMemberInput {
    pub id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteCastMemberError<E: std::error::Error> {
    #[error(transparent)]
    NotFound(#[from] NotFoundError),
    #[error("invalid cast member id: {0}")]
    InvalidId(String),
    #[error(transparent)]
    Repository(E),
}

pub struct DeleteCastMemberUseCase<R: ICastMemberRepository> {
    repo: R,
}

impl<R: ICastMemberRepository> DeleteCastMemberUseCase<R> {
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// # Errors
    /// Returns `NotFoundError` if cast member not found, or repository error.
    pub async fn execute(
        &self,
        input: DeleteCastMemberInput,
    ) -> Result<(), DeleteCastMemberError<R::Error>> {
        let cast_member_id = CastMemberId::from(&input.id)
            .map_err(|e| DeleteCastMemberError::InvalidId(e.to_string()))?;

        let mut cast_member = self
            .repo
            .find_by_id(&cast_member_id)
            .await
            .map_err(DeleteCastMemberError::Repository)?
            .ok_or_else(|| NotFoundError::new(&input.id, "CastMember"))?;

        cast_member.mark_as_deleted();

        self.repo
            .update(&cast_member)
            .await
            .map_err(DeleteCastMemberError::Repository)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::cast_member::aggregate::{CastMember, CastMemberCreateCommand};
    use crate::domain::cast_member::cast_member_type::CastMemberType;
    use crate::infrastructure::in_memory::cast_member_in_memory_repository::CastMemberInMemoryRepository;

    #[tokio::test]
    async fn should_delete_cast_member() {
        let repo = CastMemberInMemoryRepository::new();
        let cast_member = CastMember::create(CastMemberCreateCommand {
            cast_member_id: CastMemberId::new(),
            name: "John Doe".to_string(),
            cast_member_type: CastMemberType::Actor,
            created_at: Utc::now(),
        });
        repo.insert(&cast_member).await.expect("insert");

        let use_case = DeleteCastMemberUseCase::new(repo);
        use_case
            .execute(DeleteCastMemberInput {
                id: cast_member.cast_member_id().to_string(),
            })
            .await
            .expect("should delete");
    }

    #[tokio::test]
    async fn should_error_when_not_found() {
        let repo = CastMemberInMemoryRepository::new();
        let use_case = DeleteCastMemberUseCase::new(repo);

        let result = use_case
            .execute(DeleteCastMemberInput {
                id: CastMemberId::new().to_string(),
            })
            .await;

        assert!(result.is_err());
    }
}
