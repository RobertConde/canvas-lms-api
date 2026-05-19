/// Tests for v0.2.0 resource-level methods (SisImport.abort, ExternalTool.edit/delete, etc.)
use canvas_lms_api::resources::blueprint::BlueprintTemplate;
use canvas_lms_api::resources::communication_channel::CommunicationChannel;
use canvas_lms_api::resources::content_migration::ContentMigration;
use canvas_lms_api::resources::external_tool::{ExternalTool, ExternalToolParams};
use canvas_lms_api::resources::outcome::UpdateOutcomeGroupParams;
use canvas_lms_api::resources::rubric::Rubric;
use canvas_lms_api::resources::sis_import::SisImport;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---- ExternalTool ----

async fn make_tool_course(server: &MockServer, tool_id: u64) -> ExternalTool {
    // Register course endpoint
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/courses/1/external_tools/{tool_id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": tool_id,
            "name": "Tool",
            "course_id": 1
        })))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas
        .get_course(1)
        .await
        .unwrap()
        .get_external_tool(tool_id)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_external_tool_edit() {
    let server = MockServer::start().await;
    let tool = make_tool_course(&server, 5).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/external_tools/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "name": "Updated Tool",
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let params = ExternalToolParams {
        name: Some("Updated Tool".to_string()),
        ..Default::default()
    };
    let updated = tool.edit(params).await.unwrap();
    assert_eq!(updated.id, 5);
    assert_eq!(updated.name.as_deref(), Some("Updated Tool"));
}

#[tokio::test]
async fn test_external_tool_delete() {
    let server = MockServer::start().await;
    let tool = make_tool_course(&server, 5).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/external_tools/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "name": "Tool",
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let deleted = tool.delete().await.unwrap();
    assert_eq!(deleted.id, 5);
}

// ---- SisImport ----

async fn make_sis_import(server: &MockServer) -> SisImport {
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/sis_imports/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "account_id": 1,
            "workflow_state": "imported"
        })))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas
        .get_account(1)
        .await
        .unwrap()
        .get_sis_import(42)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_sis_import_abort() {
    let server = MockServer::start().await;
    let import = make_sis_import(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/accounts/1/sis_imports/42/abort"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "account_id": 1,
            "workflow_state": "aborted"
        })))
        .mount(&server)
        .await;

    let aborted = import.abort().await.unwrap();
    assert_eq!(aborted.id, 42);
    assert_eq!(aborted.workflow_state.as_deref(), Some("aborted"));
}

#[tokio::test]
async fn test_sis_import_restore_states() {
    let server = MockServer::start().await;
    let import = make_sis_import(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/accounts/1/sis_imports/42/restore_states"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99,
            "workflow_state": "queued",
            "completion": 0.0
        })))
        .mount(&server)
        .await;

    let progress = import.restore_states().await.unwrap();
    assert_eq!(progress.id, 99);
    assert_eq!(progress.workflow_state.as_deref(), Some("queued"));
}

// ---- CommunicationChannel ----

async fn make_channel(server: &MockServer) -> CommunicationChannel {
    Mock::given(method("GET"))
        .and(path("/api/v1/users/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "name": "Alice"
        })))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/users/10/communication_channels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": 55,
                "address": "alice@example.com",
                "type": "email",
                "user_id": 10,
                "workflow_state": "active"
            }
        ])))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let user = canvas
        .get_user(canvas_lms_api::resources::user::UserId::Id(10))
        .await
        .unwrap();
    user.get_communication_channels()
        .collect_all()
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
}

#[tokio::test]
async fn test_communication_channel_delete() {
    let server = MockServer::start().await;
    let channel = make_channel(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/users/10/communication_channels/55"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 55,
            "workflow_state": "deleted"
        })))
        .mount(&server)
        .await;

    let deleted = channel.delete().await.unwrap();
    assert!(deleted);
}

#[tokio::test]
async fn test_communication_channel_get_preferences() {
    let server = MockServer::start().await;
    let channel = make_channel(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/users/10/communication_channels/55/notification_preferences",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "notification_preferences": [
                {"notification": "Assignment Created", "frequency": "immediately"},
                {"notification": "Assignment Due Date", "frequency": "daily"}
            ]
        })))
        .mount(&server)
        .await;

    let prefs = channel.get_preferences().await.unwrap();
    assert_eq!(prefs.len(), 2);
}

// ---- Rubric (resource-level methods) ----

async fn make_rubric(server: &MockServer) -> Rubric {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/rubrics/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
            "title": "Essay Rubric",
            "course_id": 1
        })))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas
        .get_course(1)
        .await
        .unwrap()
        .get_rubric(3)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_rubric_delete() {
    let server = MockServer::start().await;
    let rubric = make_rubric(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/rubrics/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
            "title": "Essay Rubric",
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let deleted = rubric.delete().await.unwrap();
    assert_eq!(deleted.id, 3);
}

// ---- BlueprintTemplate ----

async fn make_blueprint(server: &MockServer) -> BlueprintTemplate {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/blueprint_templates/default"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "course_id": 1,
            "associated_course_count": 3
        })))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas
        .get_course(1)
        .await
        .unwrap()
        .get_blueprint("default")
        .await
        .unwrap()
}

#[tokio::test]
async fn test_blueprint_start_migration() {
    let server = MockServer::start().await;
    let tmpl = make_blueprint(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/blueprint_templates/1/migrations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "template_id": 1,
            "course_id": 1,
            "workflow_state": "queued"
        })))
        .mount(&server)
        .await;

    let migration = tmpl.start_migration().await.unwrap();
    assert_eq!(migration.id, 10);
    assert_eq!(migration.workflow_state.as_deref(), Some("queued"));
}

#[tokio::test]
async fn test_blueprint_get_migrations() {
    let server = MockServer::start().await;
    let tmpl = make_blueprint(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/blueprint_templates/1/migrations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "template_id": 1, "course_id": 1, "workflow_state": "completed"},
            {"id": 11, "template_id": 1, "course_id": 1, "workflow_state": "completed"}
        ])))
        .mount(&server)
        .await;

    let migrations = tmpl.get_migrations().collect_all().await.unwrap();
    assert_eq!(migrations.len(), 2);
    assert_eq!(migrations[0].id, 10);
}

// ---- OutcomeGroup (resource-level methods) ----

async fn make_outcome_group(
    server: &MockServer,
) -> canvas_lms_api::resources::outcome::OutcomeGroup {
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/outcome_groups/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20,
            "title": "Core Skills",
            "context_id": 1,
            "context_type": "Account"
        })))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas
        .get_account(1)
        .await
        .unwrap()
        .get_outcome_group(20)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_outcome_group_update() {
    let server = MockServer::start().await;
    let group = make_outcome_group(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/accounts/1/outcome_groups/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20,
            "title": "Core Skills (Updated)",
            "context_id": 1,
            "context_type": "Account"
        })))
        .mount(&server)
        .await;

    let params = UpdateOutcomeGroupParams {
        title: Some("Core Skills (Updated)".to_string()),
        ..Default::default()
    };
    let updated = group.update(params).await.unwrap();
    assert_eq!(updated.id, 20);
    assert_eq!(updated.title.as_deref(), Some("Core Skills (Updated)"));
}

#[tokio::test]
async fn test_outcome_group_create_subgroup() {
    let server = MockServer::start().await;
    let group = make_outcome_group(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/outcome_groups/20/subgroups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 21,
            "title": "Sub-Skills",
            "context_id": 1,
            "context_type": "Account"
        })))
        .mount(&server)
        .await;

    let subgroup = group.create_subgroup("Sub-Skills").await.unwrap();
    assert_eq!(subgroup.id, 21);
    assert_eq!(subgroup.title.as_deref(), Some("Sub-Skills"));
}

// ---- ContentMigration (resource-level) ----

async fn make_content_migration(server: &MockServer) -> ContentMigration {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/content_migrations/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99,
            "course_id": 1,
            "migration_type": "common_cartridge_importer",
            "workflow_state": "completed"
        })))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas
        .get_course(1)
        .await
        .unwrap()
        .get_content_migration(99)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_content_migration_get_issue() {
    let server = MockServer::start().await;
    let migration = make_content_migration(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/courses/1/content_migrations/99/migration_issues/1",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "workflow_state": "active",
            "issue_type": "warning",
            "description": "Missing attachment"
        })))
        .mount(&server)
        .await;

    let issue = migration.get_migration_issue(1).await.unwrap();
    assert_eq!(issue.id, 1);
    assert_eq!(issue.workflow_state.as_deref(), Some("active"));
}

#[tokio::test]
async fn test_content_migration_get_issues() {
    let server = MockServer::start().await;
    let migration = make_content_migration(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/courses/1/content_migrations/99/migration_issues",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "workflow_state": "active", "issue_type": "warning", "description": "Issue A"},
            {"id": 2, "workflow_state": "resolved", "issue_type": "error", "description": "Issue B"}
        ])))
        .mount(&server)
        .await;

    let issues = migration
        .get_migration_issues()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(issues.len(), 2);
    assert_eq!(issues[0].id, 1);
}
