#![cfg(feature = "new-quizzes")]

use canvas_lms_api::resources::new_quiz::NewQuizParams;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn make_course(server: &MockServer) -> canvas_lms_api::resources::course::Course {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "Test Course"
        })))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas.get_course(1).await.unwrap()
}

#[tokio::test]
async fn test_get_new_quiz() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/quiz/v1/courses/1/quizzes/abc-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "abc-123",
            "course_id": 1,
            "title": "Midterm Exam",
            "points_possible": 100.0
        })))
        .mount(&server)
        .await;

    let quiz = course.get_new_quiz("abc-123").await.unwrap();
    assert_eq!(quiz.id.as_deref(), Some("abc-123"));
    assert_eq!(quiz.title.as_deref(), Some("Midterm Exam"));
    assert_eq!(quiz.course_id, Some(1));
}

#[tokio::test]
async fn test_get_new_quizzes() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/quiz/v1/courses/1/quizzes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "q1", "course_id": 1, "title": "Quiz 1"},
            {"id": "q2", "course_id": 1, "title": "Quiz 2"}
        ])))
        .mount(&server)
        .await;

    let quizzes = course.get_new_quizzes().collect_all().await.unwrap();
    assert_eq!(quizzes.len(), 2);
    assert_eq!(quizzes[0].id.as_deref(), Some("q1"));
    assert_eq!(quizzes[0].course_id, Some(1));
    assert_eq!(quizzes[1].id.as_deref(), Some("q2"));
}

#[tokio::test]
async fn test_create_new_quiz() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/quiz/v1/courses/1/quizzes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "new-quiz-id",
            "course_id": 1,
            "title": "Final Exam",
            "points_possible": 200.0
        })))
        .mount(&server)
        .await;

    let params = NewQuizParams {
        title: Some("Final Exam".to_string()),
        points_possible: Some(200.0),
        ..Default::default()
    };
    let quiz = course.create_new_quiz(params).await.unwrap();
    assert_eq!(quiz.id.as_deref(), Some("new-quiz-id"));
    assert_eq!(quiz.title.as_deref(), Some("Final Exam"));
    assert_eq!(quiz.course_id, Some(1));
}

#[tokio::test]
async fn test_new_quiz_update() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    // First get the quiz
    Mock::given(method("GET"))
        .and(path("/api/quiz/v1/courses/1/quizzes/q99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "q99",
            "course_id": 1,
            "title": "Old Title"
        })))
        .mount(&server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/api/quiz/v1/courses/1/quizzes/q99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "q99",
            "course_id": 1,
            "title": "New Title"
        })))
        .mount(&server)
        .await;

    let quiz = course.get_new_quiz("q99").await.unwrap();
    let updated = quiz
        .update(NewQuizParams {
            title: Some("New Title".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.title.as_deref(), Some("New Title"));
}

#[tokio::test]
async fn test_new_quiz_delete() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/quiz/v1/courses/1/quizzes/q-del"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "q-del",
            "course_id": 1,
            "title": "To Delete"
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/quiz/v1/courses/1/quizzes/q-del"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "q-del",
            "course_id": 1,
            "title": "To Delete"
        })))
        .mount(&server)
        .await;

    let quiz = course.get_new_quiz("q-del").await.unwrap();
    let deleted = quiz.delete().await.unwrap();
    assert_eq!(deleted.id.as_deref(), Some("q-del"));
}

#[tokio::test]
async fn test_new_quiz_set_accommodations() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/quiz/v1/courses/1/quizzes/q-acc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "q-acc",
            "course_id": 1,
            "title": "Accessible Quiz"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/quiz/v1/courses/1/quizzes/q-acc/accommodations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "accommodations": [{"user_id": "42", "extra_time": 30}]
        })))
        .mount(&server)
        .await;

    let quiz = course.get_new_quiz("q-acc").await.unwrap();
    let result = quiz
        .set_accommodations(serde_json::json!([{"user_id": "42", "extra_time": 30}]))
        .await
        .unwrap();
    assert!(result.get("accommodations").is_some());
}
