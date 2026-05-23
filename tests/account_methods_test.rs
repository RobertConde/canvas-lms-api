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


#[tokio::test]
async fn test_account_delete() {
    let server = MockServer::start().await;
    // sub-account with parent_account_id = 2
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "name": "Sub Account",
            "parent_account_id": 2
        })))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/accounts/2/sub_accounts/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "workflow_state": "deleted"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let account = canvas.get_account(5).await.unwrap();
    let result = account.delete().await.unwrap();
    assert_eq!(result["workflow_state"], "deleted");
}

#[tokio::test]
async fn test_get_grading_periods() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/grading_periods"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "grading_periods": [
                {"id": 1, "title": "Q1", "start_date": "2024-01-01T00:00:00Z"},
                {"id": 2, "title": "Q2", "start_date": "2024-04-01T00:00:00Z"}
            ]
        })))
        .mount(&server)
        .await;

    let periods = account.get_grading_periods().collect_all().await.unwrap();
    assert_eq!(periods.len(), 2);
    assert_eq!(periods[0].title.as_deref(), Some("Q1"));
}

#[tokio::test]
async fn test_get_outcome_groups_in_context() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/outcome_groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "title": "Math Outcomes", "context_type": "Account"},
            {"id": 11, "title": "Science Outcomes", "context_type": "Account"}
        ])))
        .mount(&server)
        .await;

    let groups = account
        .get_outcome_groups_in_context()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].id, 10);
    assert_eq!(groups[0].title.as_deref(), Some("Math Outcomes"));
}

#[tokio::test]
async fn test_get_all_outcome_links_in_context() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/outcome_group_links"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"outcome": {"id": 1, "title": "Critical Thinking"}, "context_type": "Account"}
        ])))
        .mount(&server)
        .await;

    let links = account
        .get_all_outcome_links_in_context()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(links.len(), 1);
}

#[tokio::test]
async fn test_get_root_outcome_group() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/root_outcome_group"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "title": "Root Outcomes",
            "context_type": "Account",
            "context_id": 1
        })))
        .mount(&server)
        .await;

    let og = account.get_root_outcome_group().await.unwrap();
    assert_eq!(og.id, 1);
    assert_eq!(og.title.as_deref(), Some("Root Outcomes"));
}

#[tokio::test]
async fn test_get_report() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/reports/sis_export_csv/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "report": "sis_export_csv",
            "status": "complete"
        })))
        .mount(&server)
        .await;

    let result = account.get_report("sis_export_csv", 42).await.unwrap();
    assert_eq!(result["id"], 42);
    assert_eq!(result["status"], "complete");
}

#[tokio::test]
async fn test_create_notification() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/account_notifications"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "subject": "System maintenance",
            "message": "Canvas will be down for maintenance"
        })))
        .mount(&server)
        .await;

    let params = vec![
        ("account_notification[subject]".to_string(), "System maintenance".to_string()),
        ("account_notification[message]".to_string(), "Canvas will be down".to_string()),
        ("account_notification[start_at]".to_string(), "2024-01-01T00:00:00Z".to_string()),
        ("account_notification[end_at]".to_string(), "2024-01-02T00:00:00Z".to_string()),
    ];
    let result = account.create_notification(&params).await.unwrap();
    assert_eq!(result["id"], 7);
}

#[tokio::test]
async fn test_get_global_notification() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/account_notifications/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "subject": "Maintenance",
            "workflow_state": "active"
        })))
        .mount(&server)
        .await;

    let result = account.get_global_notification(7).await.unwrap();
    assert_eq!(result["id"], 7);
    assert_eq!(result["subject"], "Maintenance");
}

#[tokio::test]
async fn test_get_user_notifications() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/users/42/account_notifications"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "subject": "Welcome"},
            {"id": 2, "subject": "Update"}
        ])))
        .mount(&server)
        .await;

    let notifs = account.get_user_notifications(42).collect_all().await.unwrap();
    assert_eq!(notifs.len(), 2);
    assert_eq!(notifs[0]["subject"], "Welcome");
}

#[tokio::test]
async fn test_close_notification_for_user() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/accounts/1/users/42/account_notifications/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "workflow_state": "closed"
        })))
        .mount(&server)
        .await;

    let result = account.close_notification_for_user(42, 7).await.unwrap();
    assert_eq!(result["workflow_state"], "closed");
}

#[tokio::test]
async fn test_add_authentication_provider() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/authentication_providers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
            "auth_type": "ldap",
            "position": 1
        })))
        .mount(&server)
        .await;

    let params = vec![("auth_type".to_string(), "ldap".to_string())];
    let result = account.add_authentication_provider(&params).await.unwrap();
    assert_eq!(result["auth_type"], "ldap");
}

#[tokio::test]
async fn test_get_authentication_provider() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/authentication_providers/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
            "auth_type": "ldap",
            "position": 1
        })))
        .mount(&server)
        .await;

    let result = account.get_authentication_provider(3).await.unwrap();
    assert_eq!(result["id"], 3);
    assert_eq!(result["auth_type"], "ldap");
}

#[tokio::test]
async fn test_get_scopes() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/scopes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"resource": "courses", "verb": "GET", "scope": "url:GET|/api/v1/courses/:id"}
        ])))
        .mount(&server)
        .await;

    let scopes = account.get_scopes().collect_all().await.unwrap();
    assert_eq!(scopes.len(), 1);
    assert_eq!(scopes[0]["resource"], "courses");
}

#[tokio::test]
async fn test_query_audit_by_account() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/audit/course/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"course_id": 10, "event_type": "created", "user_id": 5}
        ])))
        .mount(&server)
        .await;

    let events = account.query_audit_by_account().collect_all().await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event_type"], "created");
}

#[tokio::test]
async fn test_get_department_level_grade_data_current() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/analytics/current/grades"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"grade": 90, "count": 15}
        ])))
        .mount(&server)
        .await;

    let result = account.get_department_level_grade_data_current().await.unwrap();
    assert!(result.is_array());
}

#[tokio::test]
async fn test_get_department_level_grade_data_completed() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/analytics/completed/grades"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"grade": 85, "count": 10}
        ])))
        .mount(&server)
        .await;

    let result = account.get_department_level_grade_data_completed().await.unwrap();
    assert!(result.is_array());
}

#[tokio::test]
async fn test_get_department_level_grade_data_with_given_term() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/analytics/terms/3/grades"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"grade": 88, "count": 12}
        ])))
        .mount(&server)
        .await;

    let result = account
        .get_department_level_grade_data_with_given_term(3)
        .await
        .unwrap();
    assert!(result.is_array());
}

#[tokio::test]
async fn test_get_department_level_participation_data_current() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/analytics/current/activity"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "participations": 350
        })))
        .mount(&server)
        .await;

    let result = account
        .get_department_level_participation_data_current()
        .await
        .unwrap();
    assert_eq!(result["participations"], 350);
}

#[tokio::test]
async fn test_get_department_level_participation_data_completed() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/analytics/completed/activity"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "participations": 200
        })))
        .mount(&server)
        .await;

    let result = account
        .get_department_level_participation_data_completed()
        .await
        .unwrap();
    assert_eq!(result["participations"], 200);
}

#[tokio::test]
async fn test_get_department_level_participation_data_with_given_term() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/analytics/terms/3/activity"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "participations": 175
        })))
        .mount(&server)
        .await;

    let result = account
        .get_department_level_participation_data_with_given_term(3)
        .await
        .unwrap();
    assert_eq!(result["participations"], 175);
}

#[tokio::test]
async fn test_get_department_level_statistics_current() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/analytics/current/statistics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "courses": 42,
            "teachers": 15
        })))
        .mount(&server)
        .await;

    let result = account.get_department_level_statistics_current().await.unwrap();
    assert_eq!(result["courses"], 42);
}

#[tokio::test]
async fn test_get_department_level_statistics_completed() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/analytics/completed/statistics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "courses": 38,
            "teachers": 12
        })))
        .mount(&server)
        .await;

    let result = account
        .get_department_level_statistics_completed()
        .await
        .unwrap();
    assert_eq!(result["courses"], 38);
}

#[tokio::test]
async fn test_get_department_level_statistics_with_given_term() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/analytics/terms/3/statistics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "courses": 30,
            "teachers": 10
        })))
        .mount(&server)
        .await;

    let result = account
        .get_department_level_statistics_with_given_term(3)
        .await
        .unwrap();
    assert_eq!(result["courses"], 30);
}

#[tokio::test]
async fn test_account_create_account() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/root_accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99, "name": "Sub Account"
        })))
        .mount(&server)
        .await;

    let sub = account
        .create_account(&[("account[name]".to_string(), "Sub Account".to_string())])
        .await
        .unwrap();
    assert_eq!(sub.id, 99);
    assert_eq!(sub.name.as_deref(), Some("Sub Account"));
}

#[tokio::test]
async fn test_account_delete_report() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/accounts/1/reports/sis_export_csv/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7, "status": "deleted"
        })))
        .mount(&server)
        .await;

    let result = account.delete_report("sis_export_csv", 7).await.unwrap();
    assert_eq!(result["id"], 7);
}

#[tokio::test]
async fn test_account_get_index_of_reports() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/reports/sis_export_csv"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "status": "complete"},
            {"id": 2, "status": "running"}
        ])))
        .mount(&server)
        .await;

    let reports: Vec<_> = account
        .get_index_of_reports("sis_export_csv")
        .collect_all()
        .await
        .unwrap();
    assert_eq!(reports.len(), 2);
}

#[tokio::test]
async fn test_account_show_account_auth_settings() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/sso_settings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sso_settings": {"login_handle_name": "Email"}
        })))
        .mount(&server)
        .await;

    let result = account.show_account_auth_settings().await.unwrap();
    assert!(result["sso_settings"].is_object());
}

#[tokio::test]
async fn test_account_update_account_auth_settings() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/accounts/1/sso_settings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sso_settings": {"login_handle_name": "Username"}
        })))
        .mount(&server)
        .await;

    let result = account
        .update_account_auth_settings(&[(
            "sso_settings[login_handle_name]".to_string(),
            "Username".to_string(),
        )])
        .await
        .unwrap();
    assert_eq!(result["sso_settings"]["login_handle_name"], "Username");
}

#[tokio::test]
async fn test_account_update_account_calendar_visibility() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/account_calendars/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42, "visible": true
        })))
        .mount(&server)
        .await;

    let result = account
        .update_account_calendar_visibility(42, &[("visible".to_string(), "true".to_string())])
        .await
        .unwrap();
    assert_eq!(result["id"], 42);
}

#[tokio::test]
async fn test_account_update_many_account_calendars_visibility() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/account_calendars"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 3
        })))
        .mount(&server)
        .await;

    let result = account
        .update_many_account_calendars_visibility(&[("visible".to_string(), "true".to_string())])
        .await
        .unwrap();
    assert_eq!(result["count"], 3);
}

#[tokio::test]
async fn test_account_update_global_notification() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/accounts/1/account_notifications/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5, "subject": "Updated Notice", "message": "Hello"
        })))
        .mount(&server)
        .await;

    let result = account
        .update_global_notification(
            5,
            &[("account_notification[subject]".to_string(), "Updated Notice".to_string())],
        )
        .await
        .unwrap();
    assert_eq!(result["id"], 5);
    assert_eq!(result["subject"], "Updated Notice");
}
