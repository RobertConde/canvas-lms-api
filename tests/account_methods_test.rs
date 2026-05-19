use canvas_lms_api::resources::account::Account;
use canvas_lms_api::resources::external_tool::ExternalToolParams;
use canvas_lms_api::resources::rubric::RubricParams;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn make_account(server: &MockServer) -> Account {
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "Test University"
        })))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas.get_account(1).await.unwrap()
}

// ---- Account Calendars ----

#[tokio::test]
async fn test_get_account_calendar() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/account_calendars/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "Test University Calendar",
            "visible": true
        })))
        .mount(&server)
        .await;

    let cal = account.get_account_calendar().await.unwrap();
    assert_eq!(cal.id, Some(1));
    assert_eq!(cal.name.as_deref(), Some("Test University Calendar"));
    assert_eq!(cal.visible, Some(true));
}

#[tokio::test]
async fn test_get_all_account_calendars() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/account_calendars"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "Dept A", "visible": true},
            {"id": 2, "name": "Dept B", "visible": false}
        ])))
        .mount(&server)
        .await;

    let cals = account
        .get_all_account_calendars()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(cals.len(), 2);
    assert_eq!(cals[0].id, Some(1));
}

// ---- External Tools ----

#[tokio::test]
async fn test_get_external_tool_on_account() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/external_tools/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "name": "Canvas Studio",
            "account_id": 1
        })))
        .mount(&server)
        .await;

    let tool = account.get_external_tool(5).await.unwrap();
    assert_eq!(tool.id, 5);
    assert_eq!(tool.name.as_deref(), Some("Canvas Studio"));
}

#[tokio::test]
async fn test_get_external_tools_on_account() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/external_tools"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 5, "name": "Tool A", "account_id": 1},
            {"id": 6, "name": "Tool B", "account_id": 1}
        ])))
        .mount(&server)
        .await;

    let tools = account.get_external_tools().collect_all().await.unwrap();
    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].id, 5);
}

#[tokio::test]
async fn test_create_external_tool_on_account() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/external_tools"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "name": "New Tool",
            "account_id": 1
        })))
        .mount(&server)
        .await;

    let params = ExternalToolParams {
        name: Some("New Tool".to_string()),
        ..Default::default()
    };
    let tool = account.create_external_tool(params).await.unwrap();
    assert_eq!(tool.id, 7);
    assert_eq!(tool.name.as_deref(), Some("New Tool"));
}

// ---- SIS Imports ----

#[tokio::test]
async fn test_get_sis_import() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/sis_imports/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "account_id": 1,
            "workflow_state": "imported"
        })))
        .mount(&server)
        .await;

    let import = account.get_sis_import(42).await.unwrap();
    assert_eq!(import.id, 42);
    assert_eq!(import.workflow_state.as_deref(), Some("imported"));
}

#[tokio::test]
async fn test_get_sis_imports() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/sis_imports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "account_id": 1, "workflow_state": "imported"},
            {"id": 2, "account_id": 1, "workflow_state": "failed"}
        ])))
        .mount(&server)
        .await;

    let imports = account.get_sis_imports().collect_all().await.unwrap();
    assert_eq!(imports.len(), 2);
    assert_eq!(imports[0].id, 1);
}

#[tokio::test]
async fn test_abort_sis_imports_pending() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/accounts/1/sis_imports/abort_all_pending"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "aborted": true
        })))
        .mount(&server)
        .await;

    let aborted = account.abort_sis_imports_pending().await.unwrap();
    assert!(aborted);
}

// ---- Rubrics ----

#[tokio::test]
async fn test_get_rubric_on_account() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/rubrics/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
            "title": "Research Paper Rubric",
            "account_id": 1
        })))
        .mount(&server)
        .await;

    let rubric = account.get_rubric(3).await.unwrap();
    assert_eq!(rubric.id, 3);
    assert_eq!(rubric.title.as_deref(), Some("Research Paper Rubric"));
}

#[tokio::test]
async fn test_get_rubrics_on_account() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/rubrics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 3, "title": "Rubric A", "account_id": 1},
            {"id": 4, "title": "Rubric B", "account_id": 1}
        ])))
        .mount(&server)
        .await;

    let rubrics = account.get_rubrics().collect_all().await.unwrap();
    assert_eq!(rubrics.len(), 2);
    assert_eq!(rubrics[0].id, 3);
}

#[tokio::test]
async fn test_create_rubric_on_account() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/rubrics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "title": "New Rubric",
            "account_id": 1
        })))
        .mount(&server)
        .await;

    let params = RubricParams {
        title: Some("New Rubric".to_string()),
        ..Default::default()
    };
    let rubric = account.create_rubric(params).await.unwrap();
    assert_eq!(rubric.id, 10);
    assert_eq!(rubric.title.as_deref(), Some("New Rubric"));
}

// ---- Outcome Groups ----

#[tokio::test]
async fn test_get_outcome_group_on_account() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/outcome_groups/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20,
            "title": "Core Skills",
            "context_id": 1,
            "context_type": "Account"
        })))
        .mount(&server)
        .await;

    let group = account.get_outcome_group(20).await.unwrap();
    assert_eq!(group.id, 20);
    assert_eq!(group.title.as_deref(), Some("Core Skills"));
}

#[tokio::test]
async fn test_get_outcome_group_links_on_account() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/outcome_group_links"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"context_id": 1, "context_type": "Account"},
            {"context_id": 1, "context_type": "Account"}
        ])))
        .mount(&server)
        .await;

    let links = account
        .get_outcome_group_links()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(links.len(), 2);
}

// ---- Content Migrations ----

#[tokio::test]
async fn test_get_content_migration_on_account() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/content_migrations/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99,
            "account_id": 1,
            "migration_type": "common_cartridge_importer",
            "workflow_state": "completed"
        })))
        .mount(&server)
        .await;

    let migration = account.get_content_migration(99).await.unwrap();
    assert_eq!(migration.id, 99);
    assert_eq!(migration.workflow_state.as_deref(), Some("completed"));
}

#[tokio::test]
async fn test_get_content_migrations_on_account() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/content_migrations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "account_id": 1, "migration_type": "common_cartridge_importer", "workflow_state": "completed"},
            {"id": 2, "account_id": 1, "migration_type": "course_copy_importer", "workflow_state": "running"}
        ])))
        .mount(&server)
        .await;

    let migrations = account
        .get_content_migrations()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(migrations.len(), 2);
    assert_eq!(migrations[0].id, 1);
}

#[tokio::test]
async fn test_get_migrators_on_account() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/content_migrations/migrators"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"type": "common_cartridge_importer", "name": "Common Cartridge", "requires_file_upload": true},
            {"type": "course_copy_importer", "name": "Course Copy", "requires_file_upload": false}
        ])))
        .mount(&server)
        .await;

    let migrators = account.get_migrators().collect_all().await.unwrap();
    assert_eq!(migrators.len(), 2);
    assert_eq!(migrators[0].name.as_deref(), Some("Common Cartridge"));
}

#[tokio::test]
async fn test_create_content_migration_on_account() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/content_migrations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 77,
            "account_id": 1,
            "migration_type": "common_cartridge_importer",
            "workflow_state": "created"
        })))
        .mount(&server)
        .await;

    let migration = account
        .create_content_migration("common_cartridge_importer", &[])
        .await
        .unwrap();
    assert_eq!(migration.id, 77);
    assert_eq!(migration.workflow_state.as_deref(), Some("created"));
}

#[tokio::test]
async fn test_get_sis_imports_running_on_account() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/sis_imports/importing"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 5, "account_id": 1, "workflow_state": "importing"},
            {"id": 6, "account_id": 1, "workflow_state": "importing"}
        ])))
        .mount(&server)
        .await;

    let imports = account
        .get_sis_imports_running()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(imports.len(), 2);
    assert_eq!(imports[0].id, 5);
}

#[tokio::test]
async fn test_create_outcome_group_on_account() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/outcome_groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 88,
            "context_id": 1,
            "context_type": "Account",
            "title": "New Outcomes"
        })))
        .mount(&server)
        .await;

    use canvas_lms_api::resources::outcome::UpdateOutcomeGroupParams;
    let group = account
        .create_outcome_group(UpdateOutcomeGroupParams {
            title: Some("New Outcomes".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(group.id, 88);
    assert_eq!(group.title.as_deref(), Some("New Outcomes"));
}
