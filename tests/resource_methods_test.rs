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

// ---- Rubric::update ----

#[tokio::test]
async fn test_rubric_update() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/rubrics/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3, "title": "Essay Rubric", "course_id": 1
        })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/rubrics/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3, "title": "Updated Rubric", "course_id": 1
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let rubric = canvas
        .get_course(1)
        .await
        .unwrap()
        .get_rubric(3)
        .await
        .unwrap();
    let updated = rubric
        .update(canvas_lms_api::resources::rubric::RubricParams {
            title: Some("Updated Rubric".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.title.as_deref(), Some("Updated Rubric"));
}

// ---- RubricAssociation and RubricAssessment ----

async fn make_rubric_association(
    server: &MockServer,
) -> canvas_lms_api::resources::rubric::RubricAssociation {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/rubric_associations/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7, "rubric_id": 3, "association_id": 1, "association_type": "Course", "course_id": 1
        })))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas
        .get_course(1)
        .await
        .unwrap()
        .get_rubric_association(7)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_rubric_association_delete() {
    let server = MockServer::start().await;
    let assoc = make_rubric_association(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/rubric_associations/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7, "rubric_id": 3, "course_id": 1
        })))
        .mount(&server)
        .await;

    let deleted = assoc.delete().await.unwrap();
    assert_eq!(deleted.id, 7);
}

#[tokio::test]
async fn test_rubric_association_update() {
    let server = MockServer::start().await;
    let assoc = make_rubric_association(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/rubric_associations/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7, "rubric_id": 3, "course_id": 1, "use_for_grading": true
        })))
        .mount(&server)
        .await;

    let params = vec![(
        "rubric_association[use_for_grading]".to_string(),
        "true".to_string(),
    )];
    let updated = assoc.update(&params).await.unwrap();
    assert_eq!(updated.use_for_grading, Some(true));
}

#[tokio::test]
async fn test_rubric_assessment_create_and_delete() {
    let server = MockServer::start().await;
    let assoc = make_rubric_association(&server).await;

    Mock::given(method("POST"))
        .and(path(
            "/api/v1/courses/1/rubric_associations/7/rubric_assessments",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99, "rubric_id": 3, "rubric_association_id": 7, "score": 10.0, "course_id": 1
        })))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(
            "/api/v1/courses/1/rubric_associations/7/rubric_assessments/99",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99, "rubric_association_id": 7, "score": 10.0, "course_id": 1
        })))
        .mount(&server)
        .await;

    let params = vec![("rubric_assessment[score]".to_string(), "10".to_string())];
    let assessment = assoc.create_rubric_assessment(&params).await.unwrap();
    assert_eq!(assessment.id, 99);
    assert_eq!(assessment.score, Some(10.0));

    let deleted = assessment.delete().await.unwrap();
    assert_eq!(deleted.id, 99);
}

#[tokio::test]
async fn test_rubric_assessment_update() {
    let server = MockServer::start().await;
    let assoc = make_rubric_association(&server).await;

    Mock::given(method("POST"))
        .and(path(
            "/api/v1/courses/1/rubric_associations/7/rubric_assessments",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 100, "rubric_id": 3, "rubric_association_id": 7, "score": 8.0, "course_id": 1
        })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path(
            "/api/v1/courses/1/rubric_associations/7/rubric_assessments/100",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 100, "rubric_association_id": 7, "score": 9.5, "course_id": 1
        })))
        .mount(&server)
        .await;

    let params = vec![("rubric_assessment[score]".to_string(), "8".to_string())];
    let assessment = assoc.create_rubric_assessment(&params).await.unwrap();

    let update_params = vec![("rubric_assessment[score]".to_string(), "9.5".to_string())];
    let updated = assessment.update(&update_params).await.unwrap();
    assert_eq!(updated.score, Some(9.5));
}

// ---- BlueprintTemplate remaining methods ----

#[tokio::test]
async fn test_blueprint_get_migration() {
    let server = MockServer::start().await;
    let tmpl = make_blueprint(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/courses/1/blueprint_templates/1/migrations/42",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42, "template_id": 1, "course_id": 1, "workflow_state": "completed"
        })))
        .mount(&server)
        .await;

    let migration = tmpl.get_migration(42).await.unwrap();
    assert_eq!(migration.id, 42);
    assert_eq!(migration.workflow_state.as_deref(), Some("completed"));
}

#[tokio::test]
async fn test_blueprint_get_unsynced_changes() {
    let server = MockServer::start().await;
    let tmpl = make_blueprint(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/courses/1/blueprint_templates/1/unsynced_changes",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"asset_id": 10, "asset_type": "assignment", "asset_name": "HW 1", "change_type": "updated"},
            {"asset_id": 11, "asset_type": "quiz", "asset_name": "Quiz 1", "change_type": "created"}
        ])))
        .mount(&server)
        .await;

    let changes = tmpl.get_unsynced_changes().collect_all().await.unwrap();
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0].asset_type.as_deref(), Some("assignment"));
}

#[tokio::test]
async fn test_blueprint_get_associated_courses() {
    let server = MockServer::start().await;
    let tmpl = make_blueprint(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/courses/1/blueprint_templates/1/associated_courses",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 5, "name": "Child Course A"},
            {"id": 6, "name": "Child Course B"}
        ])))
        .mount(&server)
        .await;

    let courses = tmpl.get_associated_courses().collect_all().await.unwrap();
    assert_eq!(courses.len(), 2);
    assert_eq!(courses[0]["id"], 5);
}

#[tokio::test]
async fn test_blueprint_update_associated_courses() {
    let server = MockServer::start().await;
    let tmpl = make_blueprint(&server).await;

    Mock::given(method("PUT"))
        .and(path(
            "/api/v1/courses/1/blueprint_templates/1/update_associations",
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"success": true})),
        )
        .mount(&server)
        .await;

    let params = vec![("course_ids_to_add[]".to_string(), "7".to_string())];
    let ok = tmpl.update_associated_courses(&params).await.unwrap();
    assert!(ok);
}

#[tokio::test]
async fn test_blueprint_migration_get_details() {
    let server = MockServer::start().await;
    let tmpl = make_blueprint(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/blueprint_templates/1/migrations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10, "template_id": 1, "course_id": 1, "workflow_state": "completed"
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/courses/1/blueprint_templates/1/migrations/10/details",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"asset_id": 20, "asset_type": "page", "asset_name": "Syllabus", "change_type": "updated"}
        ])))
        .mount(&server)
        .await;

    let migration = tmpl.start_migration().await.unwrap();
    let details = migration.get_details().collect_all().await.unwrap();
    assert_eq!(details.len(), 1);
    assert_eq!(details[0].asset_type.as_deref(), Some("page"));
}

#[tokio::test]
async fn test_blueprint_subscription_get_imports() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 2})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/2/blueprint_subscriptions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 5, "template_id": 1, "course_id": 2}
        ])))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/2/blueprint_subscriptions/5/migrations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "template_id": 1, "subscription_id": 5, "course_id": 2, "workflow_state": "completed"},
            {"id": 11, "template_id": 1, "subscription_id": 5, "course_id": 2, "workflow_state": "completed"}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let subs = canvas
        .get_course(2)
        .await
        .unwrap()
        .get_blueprint_subscriptions()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(subs.len(), 1);

    let imports = subs[0].get_imports().collect_all().await.unwrap();
    assert_eq!(imports.len(), 2);
    assert_eq!(imports[0].id, 10);
}

#[tokio::test]
async fn test_blueprint_subscription_get_import() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 2})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/2/blueprint_subscriptions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 5, "template_id": 1, "course_id": 2}
        ])))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/courses/2/blueprint_subscriptions/5/migrations/10",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10, "template_id": 1, "subscription_id": 5, "course_id": 2, "workflow_state": "completed"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let subs = canvas
        .get_course(2)
        .await
        .unwrap()
        .get_blueprint_subscriptions()
        .collect_all()
        .await
        .unwrap();
    let import = subs[0].get_import(10).await.unwrap();
    assert_eq!(import.id, 10);
    assert_eq!(import.workflow_state.as_deref(), Some("completed"));
}

// ---- OutcomeGroup remaining methods ----

async fn make_outcome_group_course(
    server: &MockServer,
) -> canvas_lms_api::resources::outcome::OutcomeGroup {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/outcome_groups/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20, "title": "Core Skills", "context_id": 1, "context_type": "Course"
        })))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas
        .get_course(1)
        .await
        .unwrap()
        .get_outcome_group(20)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_outcome_group_delete() {
    let server = MockServer::start().await;
    let group = make_outcome_group_course(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/outcome_groups/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20, "title": "Core Skills", "context_id": 1, "context_type": "Course"
        })))
        .mount(&server)
        .await;

    let deleted = group.delete().await.unwrap();
    assert_eq!(deleted.id, 20);
}

#[tokio::test]
async fn test_outcome_group_get_subgroups() {
    let server = MockServer::start().await;
    let group = make_outcome_group_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/outcome_groups/20/subgroups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 21, "title": "Sub A", "context_id": 1, "context_type": "Course"},
            {"id": 22, "title": "Sub B", "context_id": 1, "context_type": "Course"}
        ])))
        .mount(&server)
        .await;

    let subs = group.get_subgroups().collect_all().await.unwrap();
    assert_eq!(subs.len(), 2);
    assert_eq!(subs[0].id, 21);
}

#[tokio::test]
async fn test_outcome_group_get_linked_outcomes() {
    let server = MockServer::start().await;
    let group = make_outcome_group_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/outcome_groups/20/outcomes"))
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
    let group = make_outcome_group_course(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/outcome_groups/20/outcomes/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "context_id": 1, "context_type": "Course", "outcome": {"id": 5, "title": "Outcome A"}
        })))
        .mount(&server)
        .await;

    let link = group.link_outcome(5).await.unwrap();
    assert_eq!(link.context_id, Some(1));
}

#[tokio::test]
async fn test_outcome_group_unlink_outcome() {
    let server = MockServer::start().await;
    let group = make_outcome_group_course(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/outcome_groups/20/outcomes/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "context_id": 1, "context_type": "Course", "outcome": {"id": 5, "title": "Outcome A"}
        })))
        .mount(&server)
        .await;

    let link = group.unlink_outcome(5).await.unwrap();
    assert_eq!(link.context_id, Some(1));
}

// ---- Outcome::update ----

#[tokio::test]
async fn test_outcome_update() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/outcomes/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5, "title": "Original Title", "context_id": 1, "context_type": "Course"
        })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/outcomes/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5, "title": "Updated Title", "context_id": 1, "context_type": "Course"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let outcome = canvas.get_outcome(5).await.unwrap();
    let updated = outcome
        .update(canvas_lms_api::resources::outcome::UpdateOutcomeParams {
            title: Some("Updated Title".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.title.as_deref(), Some("Updated Title"));
}

// ---- ContentMigration::update ----

#[tokio::test]
async fn test_content_migration_update() {
    let server = MockServer::start().await;
    let migration = make_content_migration(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/content_migrations/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99, "course_id": 1, "migration_type": "common_cartridge_importer",
            "workflow_state": "running"
        })))
        .mount(&server)
        .await;

    let params = vec![("workflow_state".to_string(), "running".to_string())];
    let updated = migration.update(&params).await.unwrap();
    assert_eq!(updated.id, 99);
    assert_eq!(updated.workflow_state.as_deref(), Some("running"));
}

// ---- MigrationIssue::update ----

#[tokio::test]
async fn test_migration_issue_update() {
    let server = MockServer::start().await;
    let migration = make_content_migration(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/courses/1/content_migrations/99/migration_issues/1",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1, "workflow_state": "active", "issue_type": "warning",
            "content_migration_url": "/api/v1/courses/1/content_migrations/99"
        })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path(
            "/api/v1/courses/1/content_migrations/99/migration_issues/1",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1, "workflow_state": "resolved", "issue_type": "warning",
            "content_migration_url": "/api/v1/courses/1/content_migrations/99"
        })))
        .mount(&server)
        .await;

    let issue = migration.get_migration_issue(1).await.unwrap();
    let resolved = issue.update("resolved").await.unwrap();
    assert_eq!(resolved.workflow_state.as_deref(), Some("resolved"));
}
