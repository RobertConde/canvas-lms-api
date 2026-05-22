use canvas_lms_api::Canvas;
use futures::StreamExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup(server: &MockServer) -> canvas_lms_api::resources::course::Course {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"id": 1, "name": "Test Course"})),
        )
        .mount(server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas.get_course(1).await.unwrap()
}

#[tokio::test]
async fn test_course_conclude() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "delete": true
        })))
        .mount(&server)
        .await;

    let result = course.conclude().await.unwrap();
    assert!(result.is_object());
}

#[tokio::test]
async fn test_course_reset() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/reset_content"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 2,
            "name": "Copy of Test Course"
        })))
        .mount(&server)
        .await;

    let new_course = course.reset().await.unwrap();
    assert_eq!(new_course.id, 2);
}

#[tokio::test]
async fn test_course_get_settings() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/settings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "allow_student_discussion_topics": true,
            "allow_student_forum_attachments": false
        })))
        .mount(&server)
        .await;

    let settings = course.get_settings().await.unwrap();
    assert_eq!(settings["allow_student_discussion_topics"], true);
}

#[tokio::test]
async fn test_course_update_settings() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/settings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "allow_student_discussion_topics": false
        })))
        .mount(&server)
        .await;

    let updated = course
        .update_settings(&[(
            "allow_student_discussion_topics".to_string(),
            "false".to_string(),
        )])
        .await
        .unwrap();
    assert_eq!(updated["allow_student_discussion_topics"], false);
}

#[tokio::test]
async fn test_course_get_late_policy() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/late_policy"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "late_policy": {
                "id": 1,
                "course_id": 1,
                "late_submission_deduction_enabled": true,
                "late_submission_deduction": 10.0
            }
        })))
        .mount(&server)
        .await;

    let policy = course.get_late_policy().await.unwrap();
    assert!(policy.get("late_policy").is_some());
}

#[tokio::test]
async fn test_course_get_multiple_submissions() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/students/submissions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "user_id": 3, "assignment_id": 2, "grade": "A"},
            {"id": 11, "user_id": 4, "assignment_id": 2, "grade": "B"}
        ])))
        .mount(&server)
        .await;

    let subs: Vec<_> = course
        .get_multiple_submissions()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(subs.len(), 2);
    assert_eq!(subs[0].id, 10);
    assert_eq!(subs[0].course_id, Some(1));
}

#[tokio::test]
async fn test_course_enroll_user() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/enrollments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 50,
            "course_id": 1,
            "user_id": 42,
            "type": "StudentEnrollment",
            "enrollment_state": "active"
        })))
        .mount(&server)
        .await;

    let enrollment = course.enroll_user(42, "StudentEnrollment").await.unwrap();
    assert_eq!(enrollment.id, 50);
    assert_eq!(enrollment.course_id, Some(1));
}

#[tokio::test]
async fn test_course_submissions_bulk_update() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/submissions/update_grades"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99,
            "workflow_state": "queued",
            "completion": 0
        })))
        .mount(&server)
        .await;

    let progress = course
        .submissions_bulk_update(&[(
            "grade_data[3][2][posted_grade]".to_string(),
            "A".to_string(),
        )])
        .await
        .unwrap();
    assert_eq!(progress.id, 99);
    assert_eq!(progress.workflow_state.as_deref(), Some("queued"));
}
