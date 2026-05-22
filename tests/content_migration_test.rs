use canvas_lms_api::resources::content_migration::ContentMigration;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_content_migration(server: &MockServer) -> ContentMigration {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/content_migrations/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
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
        .get_content_migration(3)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_content_migration_get_migration_issue() {
    let server = MockServer::start().await;
    let migration = setup_content_migration(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/courses/1/content_migrations/3/migration_issues/1",
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
async fn test_content_migration_get_migration_issues() {
    let server = MockServer::start().await;
    let migration = setup_content_migration(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/courses/1/content_migrations/3/migration_issues",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "workflow_state": "active", "issue_type": "warning"},
            {"id": 2, "workflow_state": "resolved", "issue_type": "error"}
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
    assert_eq!(issues[1].id, 2);
}

#[tokio::test]
async fn test_content_migration_update() {
    let server = MockServer::start().await;
    let migration = setup_content_migration(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/content_migrations/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
            "course_id": 1,
            "migration_type": "common_cartridge_importer",
            "workflow_state": "running"
        })))
        .mount(&server)
        .await;

    let params = vec![("workflow_state".to_string(), "running".to_string())];
    let updated = migration.update(&params).await.unwrap();
    assert_eq!(updated.id, 3);
    assert_eq!(updated.workflow_state.as_deref(), Some("running"));
}

#[tokio::test]
async fn test_content_migration_get_progress() {
    let server = MockServer::start().await;
    let migration = setup_content_migration(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/content_migrations/3/progress"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99,
            "workflow_state": "queued",
            "completion": 0
        })))
        .mount(&server)
        .await;

    let progress = migration.get_progress().await.unwrap();
    assert_eq!(progress.id, 99);
    assert_eq!(progress.workflow_state.as_deref(), Some("queued"));
}

#[tokio::test]
async fn test_migration_issue_update() {
    let server = MockServer::start().await;
    let migration = setup_content_migration(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/courses/1/content_migrations/3/migration_issues/1",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "workflow_state": "active",
            "issue_type": "warning",
            "content_migration_url": "/api/v1/courses/1/content_migrations/3"
        })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path(
            "/api/v1/courses/1/content_migrations/3/migration_issues/1",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "workflow_state": "resolved",
            "issue_type": "warning",
            "content_migration_url": "/api/v1/courses/1/content_migrations/3"
        })))
        .mount(&server)
        .await;

    let issue = migration.get_migration_issue(1).await.unwrap();
    let resolved = issue.update("resolved").await.unwrap();
    assert_eq!(resolved.id, 1);
    assert_eq!(resolved.workflow_state.as_deref(), Some("resolved"));
}
