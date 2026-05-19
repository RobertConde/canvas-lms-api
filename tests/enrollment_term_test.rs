use canvas_lms_api::resources::enrollment_term::EnrollmentTermParams;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn term_json(id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": "Spring 2026",
        "start_at": "2026-01-01T00:00:00Z",
        "end_at": "2026-05-15T00:00:00Z",
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
async fn test_get_enrollment_term() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/terms/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(term_json(7)))
        .mount(&server)
        .await;

    let account = make_account(&server).await;
    let t = account.get_enrollment_term(7).await.unwrap();
    assert_eq!(t.id, 7);
    assert_eq!(t.name.as_deref(), Some("Spring 2026"));
}

#[tokio::test]
async fn test_get_enrollment_terms() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/terms"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([term_json(7), term_json(8)])),
        )
        .mount(&server)
        .await;

    let account = make_account(&server).await;
    let terms = account.get_enrollment_terms().collect_all().await.unwrap();
    assert_eq!(terms.len(), 2);
    assert_eq!(terms[0].account_id, Some(1));
}

#[tokio::test]
async fn test_create_enrollment_term() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/terms"))
        .respond_with(ResponseTemplate::new(200).set_body_json(term_json(9)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let account = canvas.get_account(1).await.unwrap();
    let t = account
        .create_enrollment_term(EnrollmentTermParams {
            name: Some("Summer 2026".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(t.id, 9);
}

#[tokio::test]
async fn test_enrollment_term_delete() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/terms/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(term_json(7)))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/accounts/1/terms/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(term_json(7)))
        .mount(&server)
        .await;

    let account = make_account(&server).await;
    let t = account.get_enrollment_term(7).await.unwrap();
    let deleted = t.delete().await.unwrap();
    assert_eq!(deleted.id, 7);
}

#[tokio::test]
async fn test_enrollment_term_edit() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/terms/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(term_json(7)))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/accounts/1/terms/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7, "name": "Renamed Term", "workflow_state": "active"
        })))
        .mount(&server)
        .await;

    let account = make_account(&server).await;
    let t = account.get_enrollment_term(7).await.unwrap();
    let updated = t
        .edit(EnrollmentTermParams {
            name: Some("Renamed Term".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.name.as_deref(), Some("Renamed Term"));
}
