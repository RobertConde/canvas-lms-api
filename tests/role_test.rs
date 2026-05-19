use canvas_lms_api::resources::role::RoleParams;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn role_json(id: u64, label: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "label": label,
        "role": label,
        "base_role_type": "StudentEnrollment",
        "workflow_state": "active"
    })
}

async fn make_account(server: &MockServer) -> canvas_lms_api::resources::account::Account {
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Canvas::new(&server.uri(), "token")
        .unwrap()
        .get_account(1)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_get_role() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/roles/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(role_json(3, "CustomStudent")))
        .mount(&server)
        .await;

    let account = make_account(&server).await;
    let r = account.get_role(3).await.unwrap();
    assert_eq!(r.label.as_deref(), Some("CustomStudent"));
}

#[tokio::test]
async fn test_get_roles() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/roles"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            role_json(1, "StudentEnrollment"),
            role_json(2, "TeacherEnrollment")
        ])))
        .mount(&server)
        .await;

    let account = make_account(&server).await;
    let roles = account.get_roles().collect_all().await.unwrap();
    assert_eq!(roles.len(), 2);
}

#[tokio::test]
async fn test_create_role() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/roles"))
        .respond_with(ResponseTemplate::new(200).set_body_json(role_json(10, "SuperStudent")))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let account = canvas.get_account(1).await.unwrap();
    let r = account
        .create_role(
            "SuperStudent",
            RoleParams {
                base_role_type: Some("StudentEnrollment".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(r.label.as_deref(), Some("SuperStudent"));
}

#[tokio::test]
async fn test_deactivate_role() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/accounts/1/roles/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3, "label": "CustomStudent", "workflow_state": "inactive"
        })))
        .mount(&server)
        .await;

    let account = make_account(&server).await;
    let r = account.deactivate_role(3).await.unwrap();
    assert_eq!(r.workflow_state.as_deref(), Some("inactive"));
}

#[tokio::test]
async fn test_activate_role() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/roles/3/activate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3, "label": "CustomStudent", "workflow_state": "active"
        })))
        .mount(&server)
        .await;

    let account = make_account(&server).await;
    let r = account.activate_role(3).await.unwrap();
    assert_eq!(r.workflow_state.as_deref(), Some("active"));
}

#[tokio::test]
async fn test_update_role() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/accounts/1/roles/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3, "label": "CustomStudent", "workflow_state": "active"
        })))
        .mount(&server)
        .await;

    let account = make_account(&server).await;
    let r = account
        .update_role(3, canvas_lms_api::resources::role::RoleParams::default())
        .await
        .unwrap();
    assert_eq!(r.label.as_deref(), Some("CustomStudent"));
}
