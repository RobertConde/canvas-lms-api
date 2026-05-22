use canvas_lms_api::{
    resources::account::{Account, UpdateAccountParams},
    resources::group::GroupCategoryParams,
    Canvas,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup(server: &MockServer) -> Account {
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

#[tokio::test]
async fn test_account_update() {
    let server = MockServer::start().await;
    let account = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "Updated University"
        })))
        .mount(&server)
        .await;

    let updated = account
        .update(UpdateAccountParams {
            name: Some("Updated University".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.name.as_deref(), Some("Updated University"));
}

#[tokio::test]
async fn test_account_get_subaccounts() {
    let server = MockServer::start().await;
    let account = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/sub_accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "name": "School of Arts"},
            {"id": 11, "name": "School of Science"}
        ])))
        .mount(&server)
        .await;

    let subs: Vec<_> = account.get_subaccounts().collect_all().await.unwrap();
    assert_eq!(subs.len(), 2);
    assert_eq!(subs[0].id, 10);
}

#[tokio::test]
async fn test_account_create_subaccount() {
    let server = MockServer::start().await;
    let account = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/sub_accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 12,
            "name": "New Department"
        })))
        .mount(&server)
        .await;

    let sub = account.create_subaccount("New Department").await.unwrap();
    assert_eq!(sub.id, 12);
    assert_eq!(sub.name.as_deref(), Some("New Department"));
}

#[tokio::test]
async fn test_account_get_users() {
    let server = MockServer::start().await;
    let account = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 42, "name": "Alice"},
            {"id": 43, "name": "Bob"}
        ])))
        .mount(&server)
        .await;

    let users: Vec<_> = account.get_users().collect_all().await.unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].id, 42);
}

#[tokio::test]
async fn test_account_delete_user() {
    let server = MockServer::start().await;
    let account = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/accounts/1/users/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "name": "Alice"
        })))
        .mount(&server)
        .await;

    let deleted = account.delete_user(42).await.unwrap();
    assert_eq!(deleted.id, 42);
}

#[tokio::test]
async fn test_account_get_courses() {
    let server = MockServer::start().await;
    let account = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/courses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "Math 101"},
            {"id": 2, "name": "English 101"}
        ])))
        .mount(&server)
        .await;

    let courses: Vec<_> = account.get_courses().collect_all().await.unwrap();
    assert_eq!(courses.len(), 2);
    assert_eq!(courses[0].id, 1);
}

#[tokio::test]
async fn test_account_get_groups() {
    let server = MockServer::start().await;
    let account = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 5, "name": "Study Group A"}
        ])))
        .mount(&server)
        .await;

    let groups: Vec<_> = account.get_groups().collect_all().await.unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].id, 5);
}

#[tokio::test]
async fn test_account_get_group_categories() {
    let server = MockServer::start().await;
    let account = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/group_categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "name": "Project Groups"}
        ])))
        .mount(&server)
        .await;

    let categories: Vec<_> = account.get_group_categories().collect_all().await.unwrap();
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0].id, 10);
}

#[tokio::test]
async fn test_account_create_group_category() {
    let server = MockServer::start().await;
    let account = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/group_categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 11,
            "name": "New Category"
        })))
        .mount(&server)
        .await;

    let gc = account
        .create_group_category(GroupCategoryParams {
            name: Some("New Category".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(gc.id, 11);
}

#[tokio::test]
async fn test_account_get_admins() {
    let server = MockServer::start().await;
    let account = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/admins"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "user": {"id": 42, "name": "Admin User"}, "role": "AccountAdmin"}
        ])))
        .mount(&server)
        .await;

    let admins: Vec<_> = account.get_admins().collect_all().await.unwrap();
    assert_eq!(admins.len(), 1);
}

#[tokio::test]
async fn test_account_create_admin() {
    let server = MockServer::start().await;
    let account = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/admins"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 2,
            "user_id": 99,
            "role": "AccountAdmin"
        })))
        .mount(&server)
        .await;

    let admin = account.create_admin(99).await.unwrap();
    assert_eq!(admin["user_id"], 99);
}

#[tokio::test]
async fn test_account_get_authentication_providers() {
    let server = MockServer::start().await;
    let account = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/authentication_providers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "auth_type": "ldap"},
            {"id": 2, "auth_type": "saml"}
        ])))
        .mount(&server)
        .await;

    let providers: Vec<_> = account
        .get_authentication_providers()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(providers.len(), 2);
    assert_eq!(providers[0]["auth_type"], "ldap");
}

#[tokio::test]
async fn test_account_create_user() {
    let server = MockServer::start().await;
    let account = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 55,
            "name": "New Student"
        })))
        .mount(&server)
        .await;

    let user = account
        .create_user(&[
            ("user[name]".to_string(), "New Student".to_string()),
            (
                "pseudonym[unique_id]".to_string(),
                "student@example.com".to_string(),
            ),
        ])
        .await
        .unwrap();
    assert_eq!(user.id, 55);
    assert_eq!(user.name.as_deref(), Some("New Student"));
}

#[tokio::test]
async fn test_account_get_reports() {
    let server = MockServer::start().await;
    let account = setup(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/accounts/1/reports/student_assignment_outcome_map_csv",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "report": "student_assignment_outcome_map_csv", "status": "complete"}
        ])))
        .mount(&server)
        .await;

    let reports: Vec<_> = account
        .get_reports("student_assignment_outcome_map_csv")
        .collect_all()
        .await
        .unwrap();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0]["report"], "student_assignment_outcome_map_csv");
}

#[tokio::test]
async fn test_account_create_report() {
    let server = MockServer::start().await;
    let account = setup(&server).await;

    Mock::given(method("POST"))
        .and(path(
            "/api/v1/accounts/1/reports/student_assignment_outcome_map_csv",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 2,
            "report": "student_assignment_outcome_map_csv",
            "status": "created"
        })))
        .mount(&server)
        .await;

    let report = account
        .create_report("student_assignment_outcome_map_csv", &[])
        .await
        .unwrap();
    assert_eq!(report["status"], "created");
}

#[tokio::test]
async fn test_account_get_outcome_import_status() {
    let server = MockServer::start().await;
    let account = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/outcome_imports/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "workflow_state": "succeeded",
            "processing_errors": []
        })))
        .mount(&server)
        .await;

    let status = account.get_outcome_import_status(7).await.unwrap();
    assert_eq!(status["workflow_state"], "succeeded");
}
