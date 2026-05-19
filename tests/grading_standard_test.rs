use canvas_lms_api::resources::grading_standard::{GradingSchemeEntry, GradingStandardParams};
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn standard_json(id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "title": "Letter Grade",
        "context_type": "Account",
        "context_id": 1,
        "grading_scheme": [
            {"name": "A", "value": 0.9},
            {"name": "B", "value": 0.8},
            {"name": "C", "value": 0.7}
        ]
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

// ── Account-level grading standards ──────────────────────────────────────────

#[tokio::test]
async fn test_account_get_grading_standards() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/grading_standards"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([standard_json(1), standard_json(2)])),
        )
        .mount(&server)
        .await;

    let account = make_account(&server).await;
    let standards = account.get_grading_standards().collect_all().await.unwrap();
    assert_eq!(standards.len(), 2);
    assert_eq!(standards[0].title.as_deref(), Some("Letter Grade"));
}

#[tokio::test]
async fn test_account_get_grading_standard() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/grading_standards/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(standard_json(7)))
        .mount(&server)
        .await;

    let account = make_account(&server).await;
    let s = account.get_grading_standard(7).await.unwrap();
    assert_eq!(s.id, 7);
    assert_eq!(s.title.as_deref(), Some("Letter Grade"));
}

#[tokio::test]
async fn test_account_create_grading_standard() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/grading_standards"))
        .respond_with(ResponseTemplate::new(200).set_body_json(standard_json(10)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let account = canvas.get_account(1).await.unwrap();
    let s = account
        .create_grading_standard(GradingStandardParams {
            title: "Letter Grade".into(),
            grading_scheme_entry: vec![
                GradingSchemeEntry {
                    name: Some("A".into()),
                    value: Some(0.9),
                },
                GradingSchemeEntry {
                    name: Some("B".into()),
                    value: Some(0.8),
                },
            ],
        })
        .await
        .unwrap();
    assert_eq!(s.id, 10);
}

// ── Course-level grading standards ───────────────────────────────────────────

#[tokio::test]
async fn test_course_get_grading_standards() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/grading_standards"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([standard_json(3)])),
        )
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let standards = course.get_grading_standards().collect_all().await.unwrap();
    assert_eq!(standards.len(), 1);
}

#[tokio::test]
async fn test_course_create_grading_standard() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/grading_standards"))
        .respond_with(ResponseTemplate::new(200).set_body_json(standard_json(20)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let s = course
        .create_grading_standard(GradingStandardParams {
            title: "Letter Grade".into(),
            grading_scheme_entry: vec![GradingSchemeEntry {
                name: Some("A".into()),
                value: Some(0.9),
            }],
        })
        .await
        .unwrap();
    assert_eq!(s.id, 20);
}
