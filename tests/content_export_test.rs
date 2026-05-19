use canvas_lms_api::resources::content_export::ContentExportParams;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn export_json(id: u64, ctx: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "export_type": "common_cartridge",
        "workflow_state": "created",
        "user_id": 10,
        "course_id": if ctx == "course" { serde_json::json!(1) } else { serde_json::Value::Null }
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

async fn make_course(server: &MockServer) -> canvas_lms_api::resources::course::Course {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Canvas::new(&server.uri(), "token")
        .unwrap()
        .get_course(1)
        .await
        .unwrap()
}

// ── Account content exports ───────────────────────────────────────────────────

#[tokio::test]
async fn test_account_get_content_export() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/content_exports/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(export_json(5, "account")))
        .mount(&server)
        .await;

    let account = make_account(&server).await;
    let e = account.get_content_export(5).await.unwrap();
    assert_eq!(e.id, 5);
    assert_eq!(e.export_type.as_deref(), Some("common_cartridge"));
}

#[tokio::test]
async fn test_account_get_content_exports() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/content_exports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            export_json(1, "account"),
            export_json(2, "account")
        ])))
        .mount(&server)
        .await;

    let account = make_account(&server).await;
    let exports = account.get_content_exports().collect_all().await.unwrap();
    assert_eq!(exports.len(), 2);
}

#[tokio::test]
async fn test_account_create_content_export() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/content_exports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(export_json(99, "account")))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let account = canvas.get_account(1).await.unwrap();
    let e = account
        .create_content_export(ContentExportParams {
            export_type: "common_cartridge".into(),
            skip_notifications: Some(true),
        })
        .await
        .unwrap();
    assert_eq!(e.id, 99);
    assert_eq!(e.workflow_state.as_deref(), Some("created"));
}

// ── Course content exports ────────────────────────────────────────────────────

#[tokio::test]
async fn test_course_get_content_export() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/content_exports/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(export_json(7, "course")))
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let e = course.get_content_export(7).await.unwrap();
    assert_eq!(e.id, 7);
}

#[tokio::test]
async fn test_course_get_content_exports() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/content_exports"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([export_json(3, "course")])),
        )
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let exports = course.get_content_exports().collect_all().await.unwrap();
    assert_eq!(exports.len(), 1);
}

#[tokio::test]
async fn test_course_create_content_export() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/content_exports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(export_json(88, "course")))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let e = course
        .create_content_export(ContentExportParams {
            export_type: "qti".into(),
            skip_notifications: None,
        })
        .await
        .unwrap();
    assert_eq!(e.id, 88);
}
