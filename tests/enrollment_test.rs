use canvas_lms_api::Canvas;
use futures::StreamExt;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn enrollment_json(id: u64, course_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "course_id": course_id,
        "user_id": 5,
        "type": "StudentEnrollment",
        "enrollment_state": "active"
    })
}

async fn setup(server: &MockServer) -> canvas_lms_api::resources::enrollment::Enrollment {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/enrollments"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([enrollment_json(10, 1)])),
        )
        .mount(server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let mut stream = course.get_enrollments();
    stream.next().await.unwrap().unwrap()
}

#[tokio::test]
async fn test_enrollment_accept() {
    let server = MockServer::start().await;
    let enrollment = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/enrollments/10/accept"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"success": true})))
        .mount(&server)
        .await;

    let accepted = enrollment.accept().await.unwrap();
    assert!(accepted);
}

#[tokio::test]
async fn test_enrollment_reject() {
    let server = MockServer::start().await;
    let enrollment = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/enrollments/10/reject"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"success": true})))
        .mount(&server)
        .await;

    let rejected = enrollment.reject().await.unwrap();
    assert!(rejected);
}

#[tokio::test]
async fn test_enrollment_reactivate() {
    let server = MockServer::start().await;
    let enrollment = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/enrollments/10/reactivate"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(enrollment_json(10, 1)),
        )
        .mount(&server)
        .await;

    let reactivated = enrollment.reactivate().await.unwrap();
    assert_eq!(reactivated.id, 10);
}

#[tokio::test]
async fn test_enrollment_deactivate() {
    let server = MockServer::start().await;
    let enrollment = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/enrollments/10"))
        .and(query_param("task", "conclude"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(enrollment_json(10, 1)),
        )
        .mount(&server)
        .await;

    let deactivated = enrollment.deactivate("conclude").await.unwrap();
    assert_eq!(deactivated.id, 10);
}

#[tokio::test]
async fn test_enrollment_deactivate_invalid_task() {
    let server = MockServer::start().await;
    let enrollment = setup(&server).await;

    let result = enrollment.deactivate("finish").await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        canvas_lms_api::CanvasError::BadRequest { .. }
    ));
}
