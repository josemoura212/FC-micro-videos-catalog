#![allow(clippy::unwrap_used)]

use chrono::Utc;

use catalog::application::cast_member::save_cast_member::{SaveCastMemberInput, SaveCastMemberUseCase};
use catalog::domain::cast_member::cast_member_id::CastMemberId;
use catalog::infrastructure::elasticsearch::cast_member_es_repository::CastMemberElasticSearchRepository;
use catalog::infrastructure::testing::es_helpers::EsTestHelper;

#[tokio::test]
async fn should_create_cast_member_in_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = CastMemberElasticSearchRepository::new(helper.client(), helper.index());
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
async fn should_update_cast_member_in_es() {
    let helper = EsTestHelper::start().await.expect("ES should start");
    let repo = CastMemberElasticSearchRepository::new(helper.client(), helper.index());
    let use_case = SaveCastMemberUseCase::new(repo);

    let cast_member_id = CastMemberId::new();
    use_case
        .execute(SaveCastMemberInput {
            cast_member_id: cast_member_id.to_string(),
            name: "John Doe".to_string(),
            cast_member_type: 2,
            created_at: Utc::now(),
        })
        .await
        .expect("should create");

    let output = use_case
        .execute(SaveCastMemberInput {
            cast_member_id: cast_member_id.to_string(),
            name: "Jane Director".to_string(),
            cast_member_type: 1,
            created_at: Utc::now(),
        })
        .await
        .expect("should update");

    assert_eq!(output.id, cast_member_id.to_string());
    assert!(!output.created);
}
