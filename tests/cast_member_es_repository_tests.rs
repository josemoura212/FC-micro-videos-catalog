#![allow(clippy::unwrap_used)]

use chrono::Utc;

use catalog::domain::cast_member::aggregate::{CastMember, CastMemberCreateCommand};
use catalog::domain::cast_member::cast_member_id::CastMemberId;
use catalog::domain::cast_member::cast_member_repository::ICastMemberRepository;
use catalog::domain::cast_member::cast_member_type::CastMemberType;
use catalog::domain::shared::criteria::ScopedRepository;
use catalog::infrastructure::elasticsearch::cast_member_es_repository::CastMemberElasticSearchRepository;
use catalog::infrastructure::testing::es_helpers::EsTestHelper;

fn make_cast_member(name: &str, member_type: CastMemberType) -> CastMember {
    CastMember::create(CastMemberCreateCommand {
        cast_member_id: CastMemberId::new(),
        name: name.to_string(),
        cast_member_type: member_type,
        created_at: Utc::now(),
    })
}

#[tokio::test]
async fn should_insert_and_find_by_id() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CastMemberElasticSearchRepository::new(helper.client(), helper.index());
    let cast_member = make_cast_member("John Doe", CastMemberType::Actor);

    repo.insert(&cast_member).await.unwrap();

    let found = repo.find_by_id(cast_member.cast_member_id()).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.name(), "John Doe");
    assert_eq!(found.cast_member_type(), CastMemberType::Actor);
}

#[tokio::test]
async fn should_return_none_when_not_found() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CastMemberElasticSearchRepository::new(helper.client(), helper.index());
    let found = repo.find_by_id(&CastMemberId::new()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn should_bulk_insert() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CastMemberElasticSearchRepository::new(helper.client(), helper.index());
    let cm1 = make_cast_member("John Doe", CastMemberType::Actor);
    let cm2 = make_cast_member("Jane Smith", CastMemberType::Director);

    repo.bulk_insert(&[cm1.clone(), cm2.clone()]).await.unwrap();

    let result = repo
        .find_by_ids(&[cm1.cast_member_id().clone(), cm2.cast_member_id().clone()])
        .await
        .unwrap();
    assert_eq!(result.exists.len(), 2);
}

#[tokio::test]
async fn should_find_one_by_filter() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CastMemberElasticSearchRepository::new(helper.client(), helper.index());
    let cast_member = make_cast_member("John Doe", CastMemberType::Actor);
    repo.insert(&cast_member).await.unwrap();

    let found = repo
        .find_one_by(Some(cast_member.cast_member_id()), Some(CastMemberType::Actor))
        .await
        .unwrap();
    assert!(found.is_some());

    let not_found = repo
        .find_one_by(Some(cast_member.cast_member_id()), Some(CastMemberType::Director))
        .await
        .unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn should_find_by_with_sort() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CastMemberElasticSearchRepository::new(helper.client(), helper.index());
    let cm_a = make_cast_member("AAA", CastMemberType::Actor);
    let cm_b = make_cast_member("BBB", CastMemberType::Director);
    repo.insert(&cm_b).await.unwrap();
    repo.insert(&cm_a).await.unwrap();

    let order = catalog::domain::shared::repository::SortOrder {
        field: "name".to_string(),
        direction: catalog::domain::shared::repository::SortDirection::Asc,
    };

    let results = repo.find_by(None, None, Some(&order)).await.unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].name(), "AAA");
    assert_eq!(results[1].name(), "BBB");
}

#[tokio::test]
async fn should_update_cast_member() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CastMemberElasticSearchRepository::new(helper.client(), helper.index());
    let mut cast_member = make_cast_member("John Doe", CastMemberType::Actor);
    repo.insert(&cast_member).await.unwrap();

    cast_member.change_name("Jane Director".to_string());
    cast_member.change_type(CastMemberType::Director);
    repo.update(&cast_member).await.unwrap();

    let found = repo.find_by_id(cast_member.cast_member_id()).await.unwrap().unwrap();
    assert_eq!(found.name(), "Jane Director");
    assert_eq!(found.cast_member_type(), CastMemberType::Director);
}

#[tokio::test]
async fn should_error_on_update_not_found() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CastMemberElasticSearchRepository::new(helper.client(), helper.index());
    let cast_member = make_cast_member("John Doe", CastMemberType::Actor);
    let result = repo.update(&cast_member).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn should_delete_cast_member() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CastMemberElasticSearchRepository::new(helper.client(), helper.index());
    let cast_member = make_cast_member("John Doe", CastMemberType::Actor);
    repo.insert(&cast_member).await.unwrap();

    repo.delete(cast_member.cast_member_id()).await.unwrap();

    let found = repo.find_by_id(cast_member.cast_member_id()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn should_soft_delete_with_scope() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let mut repo = CastMemberElasticSearchRepository::new(helper.client(), helper.index());
    let mut cast_member = make_cast_member("John Doe", CastMemberType::Actor);
    repo.insert(&cast_member).await.unwrap();

    cast_member.mark_as_deleted();
    repo.update(&cast_member).await.unwrap();

    repo.clear_scopes();
    let found = repo.find_by_id(cast_member.cast_member_id()).await.unwrap();
    assert!(found.is_some());

    repo.ignore_soft_deleted();
    let found = repo.find_by_id(cast_member.cast_member_id()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn should_find_all() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CastMemberElasticSearchRepository::new(helper.client(), helper.index());
    let cm1 = make_cast_member("John Doe", CastMemberType::Actor);
    let cm2 = make_cast_member("Jane Smith", CastMemberType::Director);
    repo.insert(&cm1).await.unwrap();
    repo.insert(&cm2).await.unwrap();

    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn should_exists_by_id() {
    let helper = EsTestHelper::start().await.expect("ES container should start");
    let repo = CastMemberElasticSearchRepository::new(helper.client(), helper.index());
    let cast_member = make_cast_member("John Doe", CastMemberType::Actor);
    repo.insert(&cast_member).await.unwrap();

    let result = repo.exists_by_id(&[cast_member.cast_member_id().clone()]).await.unwrap();
    assert_eq!(result.exists.len(), 1);
    assert!(result.not_exists.is_empty());

    let fake_id = CastMemberId::new();
    let result = repo.exists_by_id(&[fake_id]).await.unwrap();
    assert!(result.exists.is_empty());
    assert_eq!(result.not_exists.len(), 1);
}
