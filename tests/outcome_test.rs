use canvas_lms_api::resources::outcome::{
    OutcomeGroup, UpdateOutcomeGroupParams, UpdateOutcomeParams,
};
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_outcome_group(server: &MockServer) -> OutcomeGroup {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/root_outcome_group"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "title": "ROOT",
            "context_id": 1,
            "context_type": "Course"
        })))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas
        .get_course(1)
        .await
        .unwrap()
        .get_root_outcome_group()
        .await
        .unwrap()
}

#[tokio::test]
async fn test_outcome_update() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/outcomes/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "title": "Original Title",
            "context_id": 1,
            "context_type": "Course"
        })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/outcomes/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "title": "Updated Title",
            "context_id": 1,
            "context_type": "Course"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let outcome = canvas.get_outcome(5).await.unwrap();
    let updated = outcome
        .update(UpdateOutcomeParams {
            title: Some("Updated Title".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.id, 5);
    assert_eq!(updated.title.as_deref(), Some("Updated Title"));
}

#[tokio::test]
async fn test_outcome_group_update() {
    let server = MockServer::start().await;
    let group = setup_outcome_group(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/outcome_groups/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "title": "Updated ROOT",
            "context_id": 1,
            "context_type": "Course"
        })))
        .mount(&server)
        .await;

    let updated = group
        .update(UpdateOutcomeGroupParams {
            title: Some("Updated ROOT".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.id, 10);
    assert_eq!(updated.title.as_deref(), Some("Updated ROOT"));
}

#[tokio::test]
async fn test_outcome_group_delete() {
    let server = MockServer::start().await;
    let group = setup_outcome_group(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/outcome_groups/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "title": "ROOT",
            "context_id": 1,
            "context_type": "Course"
        })))
        .mount(&server)
        .await;

    let deleted = group.delete().await.unwrap();
    assert_eq!(deleted.id, 10);
}

#[tokio::test]
async fn test_outcome_group_get_subgroups() {
    let server = MockServer::start().await;
    let group = setup_outcome_group(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/outcome_groups/10/subgroups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 11, "title": "Sub A", "context_id": 1, "context_type": "Course"},
            {"id": 12, "title": "Sub B", "context_id": 1, "context_type": "Course"}
        ])))
        .mount(&server)
        .await;

    let subgroups = group.get_subgroups().collect_all().await.unwrap();
    assert_eq!(subgroups.len(), 2);
    assert_eq!(subgroups[0].id, 11);
    assert_eq!(subgroups[1].id, 12);
}

#[tokio::test]
async fn test_outcome_group_create_subgroup() {
    let server = MockServer::start().await;
    let group = setup_outcome_group(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/outcome_groups/10/subgroups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20,
            "title": "New Subgroup",
            "context_id": 1,
            "context_type": "Course"
        })))
        .mount(&server)
        .await;

    let subgroup = group.create_subgroup("New Subgroup").await.unwrap();
    assert_eq!(subgroup.id, 20);
    assert_eq!(subgroup.title.as_deref(), Some("New Subgroup"));
}

#[tokio::test]
async fn test_outcome_group_get_linked_outcomes() {
    let server = MockServer::start().await;
    let group = setup_outcome_group(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/outcome_groups/10/outcomes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"context_id": 1, "context_type": "Course", "outcome": {"id": 5, "title": "Outcome A"}},
            {"context_id": 1, "context_type": "Course", "outcome": {"id": 6, "title": "Outcome B"}}
        ])))
        .mount(&server)
        .await;

    let links = group.get_linked_outcomes().collect_all().await.unwrap();
    assert_eq!(links.len(), 2);
}

#[tokio::test]
async fn test_outcome_group_link_outcome() {
    let server = MockServer::start().await;
    let group = setup_outcome_group(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/outcome_groups/10/outcomes/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "context_id": 1,
            "context_type": "Course",
            "outcome": {"id": 5, "title": "Outcome A"}
        })))
        .mount(&server)
        .await;

    let link = group.link_outcome(5).await.unwrap();
    assert_eq!(link.context_id, Some(1));
}

#[tokio::test]
async fn test_outcome_group_unlink_outcome() {
    let server = MockServer::start().await;
    let group = setup_outcome_group(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/outcome_groups/10/outcomes/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "context_id": 1,
            "context_type": "Course",
            "outcome": {"id": 5, "title": "Outcome A"}
        })))
        .mount(&server)
        .await;

    let link = group.unlink_outcome(5).await.unwrap();
    assert_eq!(link.context_id, Some(1));
}

#[tokio::test]
async fn test_outcome_group_import_outcome_group() {
    let server = MockServer::start().await;
    let group = setup_outcome_group(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/outcome_groups/10/import"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99,
            "title": "Imported Group",
            "context_id": 1,
            "context_type": "Course"
        })))
        .mount(&server)
        .await;

    let imported = group.import_outcome_group(50).await.unwrap();
    assert_eq!(imported.id, 99);
    assert_eq!(imported.title.as_deref(), Some("Imported Group"));
}

// ---- OutcomeImport (Batch 6) ----

#[tokio::test]
async fn test_account_import_outcomes() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/outcome_imports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "account_id": 1,
            "workflow_state": "created",
            "progress": 0.0
        })))
        .mount(&server)
        .await;

    let canvas = canvas_lms_api::Canvas::new(&server.uri(), "token").unwrap();
    let account = canvas.get_account(1).await.unwrap();
    let import = account.import_outcomes(&[]).await.unwrap();
    assert_eq!(import.id, 7);
    assert_eq!(import.workflow_state.as_deref(), Some("created"));
    assert_eq!(import.account_id, Some(1));
}

#[tokio::test]
async fn test_outcome_import_get_progress() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/outcome_imports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "account_id": 1,
            "workflow_state": "created"
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/outcome_imports/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "account_id": 1,
            "workflow_state": "succeeded"
        })))
        .mount(&server)
        .await;

    let canvas = canvas_lms_api::Canvas::new(&server.uri(), "token").unwrap();
    let account = canvas.get_account(1).await.unwrap();
    let import = account.import_outcomes(&[]).await.unwrap();
    let progress = import.get_progress().await.unwrap();
    assert_eq!(progress["id"], 7);
    assert_eq!(progress["workflow_state"], "succeeded");
}

#[tokio::test]
async fn test_outcome_group_link_new() {
    let server = MockServer::start().await;
    let group = setup_outcome_group(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/outcome_groups/10/outcomes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "context_id": 1,
            "context_type": "Course",
            "outcome": {"id": 20, "title": "New Outcome"},
            "outcome_group": {"id": 10, "context_id": 1, "context_type": "Course"}
        })))
        .mount(&server)
        .await;

    let link = group.link_new("New Outcome", &[]).await.unwrap();
    assert_eq!(link.outcome.as_ref().and_then(|v| v["id"].as_u64()), Some(20));
}

#[tokio::test]
async fn test_outcome_link_get_outcome() {
    let server = MockServer::start().await;
    let group = setup_outcome_group(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/outcome_groups/10/outcomes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([{
            "context_id": 1,
            "context_type": "Course",
            "outcome": {"id": 5, "title": "Linked Outcome"},
            "outcome_group": {"id": 10, "context_id": 1, "context_type": "Course"}
        }])))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/outcomes/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "title": "Linked Outcome"
        })))
        .mount(&server)
        .await;

    let links = group.get_linked_outcomes().collect_all().await.unwrap();
    let outcome = links[0].get_outcome().await.unwrap();
    assert_eq!(outcome.id, 5);
    assert_eq!(outcome.title.as_deref(), Some("Linked Outcome"));
}

#[tokio::test]
async fn test_outcome_link_get_outcome_group() {
    let server = MockServer::start().await;
    let group = setup_outcome_group(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/outcome_groups/10/outcomes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([{
            "context_id": 1,
            "context_type": "Course",
            "outcome": {"id": 5, "title": "Linked Outcome"},
            "outcome_group": {"id": 10, "context_id": 1, "context_type": "Course", "title": "ROOT"}
        }])))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/outcome_groups/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "title": "ROOT",
            "context_id": 1,
            "context_type": "Course"
        })))
        .mount(&server)
        .await;

    let links = group.get_linked_outcomes().collect_all().await.unwrap();
    let og = links[0].get_outcome_group().await.unwrap();
    assert_eq!(og.id, 10);
    assert_eq!(og.title.as_deref(), Some("ROOT"));
}
