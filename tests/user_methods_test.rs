use canvas_lms_api::resources::user::UserId;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn make_user(server: &MockServer) -> canvas_lms_api::resources::user::User {
    Mock::given(method("GET"))
        .and(path("/api/v1/users/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "name": "Alice"
        })))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas.get_user(UserId::Id(42)).await.unwrap()
}

#[tokio::test]
async fn test_user_get_courses() {
    let server = MockServer::start().await;
    let user = make_user(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/courses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "Course A"},
            {"id": 2, "name": "Course B"}
        ])))
        .mount(&server)
        .await;

    let courses = user.get_courses().collect_all().await.unwrap();

    assert_eq!(courses.len(), 2);
    assert_eq!(courses[0].id, 1);
    assert_eq!(courses[1].id, 2);
}

#[tokio::test]
async fn test_user_get_enrollments() {
    let server = MockServer::start().await;
    let user = make_user(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/enrollments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 101, "course_id": 1, "user_id": 42},
            {"id": 102, "course_id": 2, "user_id": 42}
        ])))
        .mount(&server)
        .await;

    let enrollments = user.get_enrollments().collect_all().await.unwrap();

    assert_eq!(enrollments.len(), 2);
    assert_eq!(enrollments[0].id, 101);
    assert_eq!(enrollments[1].id, 102);
}
