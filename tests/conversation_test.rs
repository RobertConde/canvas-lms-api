use canvas_lms_api::resources::conversation::ConversationParams;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn conv_json(id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "subject": "Test Subject",
        "workflow_state": "read",
        "message_count": 1
    })
}

#[tokio::test]
async fn test_get_conversation() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/conversations/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(conv_json(42)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let c = canvas.get_conversation(42).await.unwrap();
    assert_eq!(c.id, 42);
    assert_eq!(c.subject.as_deref(), Some("Test Subject"));
}

#[tokio::test]
async fn test_get_conversations() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/conversations"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([conv_json(1), conv_json(2)])),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let convs = canvas.get_conversations().collect_all().await.unwrap();
    assert_eq!(convs.len(), 2);
}

#[tokio::test]
async fn test_create_conversation() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/conversations"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!([
            { "id": 99, "subject": "Hello", "workflow_state": "unread" }
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let c = canvas
        .create_conversation(
            &["1", "2"],
            "Hello there",
            ConversationParams {
                subject: Some("Hello".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(c.id, 99);
    assert_eq!(c.subject.as_deref(), Some("Hello"));
}

#[tokio::test]
async fn test_conversation_add_message() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/conversations/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(conv_json(10)))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/conversations/10/add_message"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10, "subject": "Test Subject", "workflow_state": "read"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let conv = canvas.get_conversation(10).await.unwrap();
    let updated = conv.add_message("A new reply").await.unwrap();
    assert_eq!(updated.id, 10);
}

#[tokio::test]
async fn test_conversation_delete() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/conversations/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(conv_json(10)))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/conversations/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10, "workflow_state": "read"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let conv = canvas.get_conversation(10).await.unwrap();
    let deleted = conv.delete().await.unwrap();
    assert_eq!(deleted.id, 10);
}

#[tokio::test]
async fn test_conversation_add_recipients() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/conversations/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(conv_json(10)))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/conversations/10/add_recipients"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10, "subject": "Test Subject", "workflow_state": "read"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let conv = canvas.get_conversation(10).await.unwrap();
    let updated = conv.add_recipients(&["5", "6"]).await.unwrap();
    assert_eq!(updated.id, 10);
}

#[tokio::test]
async fn test_conversation_delete_messages() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/conversations/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(conv_json(10)))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/conversations/10/remove_messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10, "message_count": 0
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let conv = canvas.get_conversation(10).await.unwrap();
    let result = conv.delete_messages(&[101, 102]).await.unwrap();
    assert_eq!(result["id"], 10);
}

#[tokio::test]
async fn test_conversation_set_workflow_state() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/conversations/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(conv_json(10)))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/conversations/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10, "subject": "Test Subject", "workflow_state": "archived"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let conv = canvas.get_conversation(10).await.unwrap();
    let updated = conv.set_workflow_state("archived").await.unwrap();
    assert_eq!(updated.workflow_state.as_deref(), Some("archived"));
}
