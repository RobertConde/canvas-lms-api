use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn quiz_json(id: u64, course_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "course_id": course_id,
        "title": "Quiz 1",
        "quiz_type": "assignment",
        "published": true
    })
}

fn group_json(id: u64, quiz_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "quiz_id": quiz_id,
        "name": "Group A",
        "pick_count": 3,
        "question_points": 2.0,
        "position": 1
    })
}

/// Fetch a `QuizGroup` via `Quiz::get_quiz_group`, which injects requester,
/// course_id and quiz_id so that group-level operations are available.
async fn setup_group(server: &MockServer) -> canvas_lms_api::resources::quiz::QuizGroup {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/quizzes/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(quiz_json(5, 1)))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/quizzes/5/groups/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(group_json(10, 5)))
        .mount(server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let quiz = course.get_quiz(5).await.unwrap();
    quiz.get_quiz_group(10).await.unwrap()
}

#[tokio::test]
async fn test_quiz_group_update() {
    let server = MockServer::start().await;
    let group = setup_group(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/quizzes/5/groups/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "quiz_id": 5,
            "name": "Updated Group",
            "pick_count": 5,
            "question_points": 3.0,
            "position": 1
        })))
        .mount(&server)
        .await;

    let updated = group
        .update(&[
            (
                "quiz_groups[][name]".to_string(),
                "Updated Group".to_string(),
            ),
            ("quiz_groups[][pick_count]".to_string(), "5".to_string()),
        ])
        .await
        .unwrap();
    assert_eq!(updated.id, 10);
    assert_eq!(updated.name.as_deref(), Some("Updated Group"));
    assert_eq!(updated.course_id, Some(1));
    assert_eq!(updated.quiz_id, Some(5));
}

#[tokio::test]
async fn test_quiz_group_delete() {
    let server = MockServer::start().await;
    let group = setup_group(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/quizzes/5/groups/10"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    group.delete().await.unwrap();
}

#[tokio::test]
async fn test_quiz_group_reorder() {
    let server = MockServer::start().await;
    let group = setup_group(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/quizzes/5/groups/10/reorder"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    group
        .reorder_question_group(&[
            ("order[][type]".to_string(), "question".to_string()),
            ("order[][id]".to_string(), "42".to_string()),
        ])
        .await
        .unwrap();
}
