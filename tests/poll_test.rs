use canvas_lms_api::Canvas;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn poll_json(id: u64) -> serde_json::Value {
    serde_json::json!({ "polls": [{"id": id, "question": "Favorite color?"}] })
}

fn choice_json(id: u64, poll_id: u64) -> serde_json::Value {
    serde_json::json!({ "poll_choices": [{"id": id, "poll_id": poll_id, "text": "Blue"}] })
}

fn session_json(id: u64, poll_id: u64) -> serde_json::Value {
    serde_json::json!({
        "poll_sessions": [{"id": id, "poll_id": poll_id, "course_id": 10, "is_published": false}]
    })
}

#[tokio::test]
async fn test_get_poll() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(poll_json(1)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let poll = canvas.get_poll(1).await.unwrap();
    assert_eq!(poll.id, 1);
    assert_eq!(poll.question.as_deref(), Some("Favorite color?"));
}

#[tokio::test]
async fn test_get_polls() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!([{"id": 1, "question": "A"}, {"id": 2, "question": "B"}]),
        ))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let polls = canvas.get_polls().collect_all().await.unwrap();
    assert_eq!(polls.len(), 2);
    assert_eq!(polls[0].id, 1);
}

#[tokio::test]
async fn test_create_poll() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/polls"))
        .and(body_string_contains("polls%5B%5D%5Bquestion%5D=My+Poll"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"polls": [{"id": 5, "question": "My Poll"}]})),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let poll = canvas
        .create_poll(canvas_lms_api::resources::poll::CreatePollParams {
            question: "My Poll".into(),
            description: None,
        })
        .await
        .unwrap();
    assert_eq!(poll.id, 5);
    assert_eq!(poll.question.as_deref(), Some("My Poll"));
}

#[tokio::test]
async fn test_poll_delete() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(poll_json(1)))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/polls/1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let poll = canvas.get_poll(1).await.unwrap();
    poll.delete().await.unwrap();
}

#[tokio::test]
async fn test_poll_get_choices() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(poll_json(1)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1/poll_choices"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "poll_id": 1, "text": "Blue"},
            {"id": 11, "poll_id": 1, "text": "Red"}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let poll = canvas.get_poll(1).await.unwrap();
    let choices = poll.get_choices().collect_all().await.unwrap();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].text.as_deref(), Some("Blue"));
}

#[tokio::test]
async fn test_poll_get_choice() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(poll_json(1)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1/poll_choices/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(choice_json(10, 1)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let poll = canvas.get_poll(1).await.unwrap();
    let choice = poll.get_choice(10).await.unwrap();
    assert_eq!(choice.id, 10);
    assert_eq!(choice.text.as_deref(), Some("Blue"));
}

#[tokio::test]
async fn test_poll_create_session() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(poll_json(1)))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/polls/1/poll_sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(session_json(20, 1)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let poll = canvas.get_poll(1).await.unwrap();
    let session = poll
        .create_session(canvas_lms_api::resources::poll::PollSessionParams {
            course_id: 10,
            course_section_id: None,
            has_public_results: None,
        })
        .await
        .unwrap();
    assert_eq!(session.id, 20);
    assert_eq!(session.course_id, Some(10));
}

#[tokio::test]
async fn test_poll_update() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(poll_json(1)))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/polls/1"))
        .and(body_string_contains(
            "polls%5B%5D%5Bquestion%5D=New+Question",
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"polls": [{"id": 1, "question": "New Question"}]}),
            ),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let poll = canvas.get_poll(1).await.unwrap();
    let updated = poll
        .update(canvas_lms_api::resources::poll::CreatePollParams {
            question: "New Question".into(),
            description: None,
        })
        .await
        .unwrap();
    assert_eq!(updated.question.as_deref(), Some("New Question"));
}

#[tokio::test]
async fn test_poll_get_sessions() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(poll_json(1)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1/poll_sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 20, "poll_id": 1, "course_id": 10},
            {"id": 21, "poll_id": 1, "course_id": 11}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let poll = canvas.get_poll(1).await.unwrap();
    let sessions = poll.get_sessions().collect_all().await.unwrap();
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].id, 20);
    assert_eq!(sessions[1].id, 21);
}

#[tokio::test]
async fn test_poll_create_choice() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(poll_json(1)))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/polls/1/poll_choices"))
        .and(body_string_contains("poll_choice%5B%5D%5Btext%5D=Green"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"poll_choices": [{"id": 30, "poll_id": 1, "text": "Green"}]}),
        ))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let poll = canvas.get_poll(1).await.unwrap();
    let choice = poll
        .create_choice(canvas_lms_api::resources::poll::PollChoiceParams {
            text: "Green".into(),
            is_correct: None,
            position: None,
        })
        .await
        .unwrap();
    assert_eq!(choice.id, 30);
    assert_eq!(choice.text.as_deref(), Some("Green"));
}

#[tokio::test]
async fn test_poll_choice_update() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(poll_json(1)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1/poll_choices/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(choice_json(10, 1)))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/polls/1/poll_choices/10"))
        .and(body_string_contains("poll_choice%5B%5D%5Btext%5D=Purple"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"poll_choices": [{"id": 10, "poll_id": 1, "text": "Purple"}]}),
        ))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let poll = canvas.get_poll(1).await.unwrap();
    let choice = poll.get_choice(10).await.unwrap();
    let updated = choice
        .update(canvas_lms_api::resources::poll::PollChoiceParams {
            text: "Purple".into(),
            is_correct: None,
            position: None,
        })
        .await
        .unwrap();
    assert_eq!(updated.text.as_deref(), Some("Purple"));
}

#[tokio::test]
async fn test_poll_choice_delete() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(poll_json(1)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1/poll_choices/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(choice_json(10, 1)))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/polls/1/poll_choices/10"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let poll = canvas.get_poll(1).await.unwrap();
    let choice = poll.get_choice(10).await.unwrap();
    choice.delete().await.unwrap();
}

#[tokio::test]
async fn test_poll_session_update() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(poll_json(1)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1/poll_sessions/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(session_json(20, 1)))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/polls/1/poll_sessions/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"poll_sessions": [{"id": 20, "poll_id": 1, "course_id": 99}]}),
        ))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let poll = canvas.get_poll(1).await.unwrap();
    let session = poll.get_session(20).await.unwrap();
    let updated = session
        .update(canvas_lms_api::resources::poll::PollSessionParams {
            course_id: 99,
            course_section_id: None,
            has_public_results: None,
        })
        .await
        .unwrap();
    assert_eq!(updated.course_id, Some(99));
}

#[tokio::test]
async fn test_poll_session_delete() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(poll_json(1)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1/poll_sessions/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(session_json(20, 1)))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/polls/1/poll_sessions/20"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let poll = canvas.get_poll(1).await.unwrap();
    let session = poll.get_session(20).await.unwrap();
    session.delete().await.unwrap();
}

#[tokio::test]
async fn test_poll_session_get_submission() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(poll_json(1)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1/poll_sessions/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(session_json(20, 1)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1/poll_sessions/20/poll_submissions/50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "poll_submissions": [{"id": 50, "poll_session_id": 20, "poll_choice_id": 10}]
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let poll = canvas.get_poll(1).await.unwrap();
    let session = poll.get_session(20).await.unwrap();
    let submission = session.get_submission(50).await.unwrap();
    assert_eq!(submission.id, 50);
    assert_eq!(submission.poll_choice_id, Some(10));
}

#[tokio::test]
async fn test_poll_session_create_submission() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(poll_json(1)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1/poll_sessions/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(session_json(20, 1)))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/polls/1/poll_sessions/20/poll_submissions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "poll_submissions": [{"id": 60, "poll_session_id": 20, "poll_choice_id": 10}]
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let poll = canvas.get_poll(1).await.unwrap();
    let session = poll.get_session(20).await.unwrap();
    let submission = session
        .create_submission(canvas_lms_api::resources::poll::PollSubmissionParams {
            poll_choice_id: 10,
        })
        .await
        .unwrap();
    assert_eq!(submission.id, 60);
    assert_eq!(submission.poll_session_id, Some(20));
}

#[tokio::test]
async fn test_poll_session_open_close() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(poll_json(1)))
        .up_to_n_times(2)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1/poll_sessions/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(session_json(20, 1)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1/poll_sessions/20/open"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"poll_sessions": [{"id": 20, "poll_id": 1, "is_published": true}]}),
        ))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/polls/1/poll_sessions/20/close"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"poll_sessions": [{"id": 20, "poll_id": 1, "is_published": false}]}),
        ))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let poll = canvas.get_poll(1).await.unwrap();
    let session = poll.get_session(20).await.unwrap();

    let opened = session.open().await.unwrap();
    assert_eq!(opened.is_published, Some(true));

    let closed = opened.close().await.unwrap();
    assert_eq!(closed.is_published, Some(false));
}
