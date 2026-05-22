use canvas_lms_api::{
    resources::{
        params::quiz_params::CreateQuizParams,
        quiz::{QuizQuestionParams, UpdateQuizSubmissionParams},
    },
    Canvas,
};
use futures::StreamExt;
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

fn question_json(id: u64, quiz_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "quiz_id": quiz_id,
        "question_name": "Q1",
        "question_type": "multiple_choice_question",
        "points_possible": 1.0
    })
}

fn submission_json(id: u64, quiz_id: u64, user_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "quiz_id": quiz_id,
        "user_id": user_id,
        "attempt": 1,
        "workflow_state": "untaken",
        "validation_token": "abc123"
    })
}

fn wrapped_submission(id: u64, quiz_id: u64, user_id: u64) -> serde_json::Value {
    serde_json::json!({
        "quiz_submissions": [submission_json(id, quiz_id, user_id)]
    })
}

async fn setup(server: &MockServer) -> canvas_lms_api::resources::quiz::Quiz {
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

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    course.get_quiz(5).await.unwrap()
}

#[tokio::test]
async fn test_quiz_edit() {
    let server = MockServer::start().await;
    let quiz = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/quizzes/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "course_id": 1,
            "title": "Updated Quiz"
        })))
        .mount(&server)
        .await;

    let updated = quiz
        .edit(CreateQuizParams {
            title: "Updated Quiz".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.title.as_deref(), Some("Updated Quiz"));
    assert_eq!(updated.course_id, Some(1));
}

#[tokio::test]
async fn test_quiz_delete() {
    let server = MockServer::start().await;
    let quiz = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/quizzes/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(quiz_json(5, 1)))
        .mount(&server)
        .await;

    let deleted = quiz.delete().await.unwrap();
    assert_eq!(deleted.id, 5);
}

#[tokio::test]
async fn test_quiz_create_question() {
    let server = MockServer::start().await;
    let quiz = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/quizzes/5/questions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(question_json(10, 5)))
        .mount(&server)
        .await;

    let q = quiz
        .create_question(QuizQuestionParams {
            question_name: Some("Q1".to_string()),
            question_type: Some("multiple_choice_question".to_string()),
            points_possible: Some(1.0),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(q.id, 10);
    assert_eq!(q.quiz_id, Some(5));
    assert_eq!(q.course_id, Some(1));
}

#[tokio::test]
async fn test_quiz_get_question() {
    let server = MockServer::start().await;
    let quiz = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/quizzes/5/questions/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(question_json(10, 5)))
        .mount(&server)
        .await;

    let q = quiz.get_question(10).await.unwrap();
    assert_eq!(q.id, 10);
    assert_eq!(q.quiz_id, Some(5));
    assert_eq!(q.course_id, Some(1));
}

#[tokio::test]
async fn test_quiz_get_questions() {
    let server = MockServer::start().await;
    let quiz = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/quizzes/5/questions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            question_json(10, 5),
            question_json(11, 5)
        ])))
        .mount(&server)
        .await;

    let questions: Vec<_> = quiz
        .get_questions()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(questions.len(), 2);
    assert_eq!(questions[0].id, 10);
    assert_eq!(questions[0].course_id, Some(1));
}

#[tokio::test]
async fn test_quiz_create_submission() {
    let server = MockServer::start().await;
    let quiz = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/quizzes/5/submissions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(wrapped_submission(20, 5, 3)))
        .mount(&server)
        .await;

    let sub = quiz.create_submission().await.unwrap();
    assert_eq!(sub.id, 20);
    assert_eq!(sub.quiz_id, Some(5));
    assert_eq!(sub.course_id, Some(1));
}

#[tokio::test]
async fn test_quiz_get_submission() {
    let server = MockServer::start().await;
    let quiz = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/quizzes/5/submissions/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(wrapped_submission(20, 5, 3)))
        .mount(&server)
        .await;

    let sub = quiz.get_submission(20).await.unwrap();
    assert_eq!(sub.id, 20);
    assert_eq!(sub.quiz_id, Some(5));
}

#[tokio::test]
async fn test_quiz_get_submissions() {
    let server = MockServer::start().await;
    let quiz = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/quizzes/5/submissions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            submission_json(20, 5, 3),
            submission_json(21, 5, 4)
        ])))
        .mount(&server)
        .await;

    let subs: Vec<_> = quiz
        .get_submissions()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(subs.len(), 2);
    assert_eq!(subs[0].id, 20);
    assert_eq!(subs[0].course_id, Some(1));
}

#[tokio::test]
async fn test_quiz_get_statistics() {
    let server = MockServer::start().await;
    let quiz = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/quizzes/5/statistics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "quiz_statistics": [{"id": 5, "multiple_attempts_exist": false}]
        })))
        .mount(&server)
        .await;

    let stats = quiz.get_statistics().await.unwrap();
    assert!(stats.get("quiz_statistics").is_some());
}

async fn setup_with_submission(
    server: &MockServer,
) -> canvas_lms_api::resources::quiz::QuizSubmission {
    let quiz = setup(server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/quizzes/5/submissions/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(wrapped_submission(20, 5, 3)))
        .mount(server)
        .await;

    quiz.get_submission(20).await.unwrap()
}

#[tokio::test]
async fn test_quiz_submission_complete() {
    let server = MockServer::start().await;
    let sub = setup_with_submission(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/quizzes/5/submissions/20/complete"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "quiz_submissions": [{
                "id": 20,
                "quiz_id": 5,
                "user_id": 3,
                "attempt": 1,
                "workflow_state": "complete"
            }]
        })))
        .mount(&server)
        .await;

    let completed = sub.complete("abc123").await.unwrap();
    assert_eq!(completed.id, 20);
    assert_eq!(completed.workflow_state.as_deref(), Some("complete"));
}

#[tokio::test]
async fn test_quiz_submission_get_times() {
    let server = MockServer::start().await;
    let sub = setup_with_submission(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/quizzes/5/submissions/20/time"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "end_at": "2024-01-01T12:00:00Z",
            "time_left": 300
        })))
        .mount(&server)
        .await;

    let times = sub.get_times().await.unwrap();
    assert_eq!(times["time_left"], 300);
}

#[tokio::test]
async fn test_quiz_submission_get_questions() {
    let server = MockServer::start().await;
    let sub = setup_with_submission(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/quiz_submissions/20/questions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "quiz_submission_questions": [
                {"id": 100, "question_name": "Q1"},
                {"id": 101, "question_name": "Q2"}
            ]
        })))
        .mount(&server)
        .await;

    let questions = sub.get_submission_questions().await.unwrap();
    assert_eq!(questions.len(), 2);
}

#[tokio::test]
async fn test_quiz_submission_update_score() {
    let server = MockServer::start().await;
    let sub = setup_with_submission(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/quizzes/5/submissions/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "quiz_submissions": [{
                "id": 20,
                "quiz_id": 5,
                "user_id": 3,
                "fudge_points": 2.0
            }]
        })))
        .mount(&server)
        .await;

    let updated = sub
        .update_score_and_comments(UpdateQuizSubmissionParams {
            fudge_points: Some(2.0),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.id, 20);
    assert_eq!(updated.fudge_points, Some(2.0));
}

#[tokio::test]
async fn test_quiz_set_extensions() {
    let server = MockServer::start().await;
    let quiz = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/quizzes/5/extensions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "quiz_extensions": [{"user_id": 10, "extra_attempts": 1}]
        })))
        .mount(&server)
        .await;

    let result = quiz
        .set_extensions(&[
            ("quiz_extensions[][user_id]".to_string(), "10".to_string()),
            ("quiz_extensions[][extra_attempts]".to_string(), "1".to_string()),
        ])
        .await
        .unwrap();
    assert!(result.get("quiz_extensions").is_some());
}
