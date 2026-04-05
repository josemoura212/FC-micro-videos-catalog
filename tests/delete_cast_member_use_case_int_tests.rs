#![allow(clippy::unwrap_used)]

use chrono::Utc;

use catalog::application::cast_member::delete_cast_member::{DeleteCastMemberInput, DeleteCastMemberUseCase};
use catalog::domain::cast_member::aggregate::{CastMember, CastMemberCreateCommand};
use catalog::domain::cast_member::cast_member_id::CastMemberId;
use catalog::domain::cast_member::cast_member_repository::ICastMemberRepository;
use catalog::domain::cast_member::cast_member_type::CastMemberType;
use catalog::infrastructure::elasticsearch::cast_member_es_repository::CastMemberElasticSearchRepository;
use catalog::infrastructure::testing::es_helpers::EsTestHelper;

#[tokio::test]
async fn should_soft_delete_cast_member_in_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = CastMemberElasticSearchRepository::new(helper.client(), helper.index());

    let cast_member = CastMember::create(CastMemberCreateCommand {
        cast_member_id: CastMemberId::new(),
        name: "John Doe".to_string(),
        cast_member_type: CastMemberType::Actor,
        created_at: Utc::now(),
    });
    repo.insert(&cast_member).await.unwrap();

    let use_case = DeleteCastMemberUseCase::new(repo);
    use_case
        .execute(DeleteCastMemberInput {
            id: cast_member.cast_member_id().to_string(),
        })
        .await
        .expect("should delete");
}

#[tokio::test]
async fn should_error_when_deleting_not_found() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = CastMemberElasticSearchRepository::new(helper.client(), helper.index());
    let use_case = DeleteCastMemberUseCase::new(repo);

    let result = use_case
        .execute(DeleteCastMemberInput {
            id: CastMemberId::new().to_string(),
        })
        .await;

    assert!(result.is_err());
}
