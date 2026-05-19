use canvas_lms_api::resources::user::UserId;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_get_communication_channels() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "name": "Alice"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/10/communication_channels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": 55,
                "address": "alice@example.com",
                "type": "email",
                "user_id": 10,
                "workflow_state": "active"
            },
            {
                "id": 56,
                "address": "+15551234567",
                "type": "sms",
                "user_id": 10,
                "workflow_state": "active"
            }
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let user = canvas.get_user(UserId::Id(10)).await.unwrap();
    let channels = user
        .get_communication_channels()
        .collect_all()
        .await
        .unwrap();

    assert_eq!(channels.len(), 2);
    assert_eq!(channels[0].id, 55);
    assert_eq!(channels[0].address.as_deref(), Some("alice@example.com"));
    assert_eq!(channels[0].channel_type.as_deref(), Some("email"));
    assert_eq!(channels[1].id, 56);
    assert_eq!(channels[1].channel_type.as_deref(), Some("sms"));
}

#[tokio::test]
async fn test_create_communication_channel() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "name": "Alice"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v1/users/10/communication_channels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 57,
            "address": "alice@newdomain.com",
            "type": "email",
            "user_id": 10,
            "workflow_state": "unconfirmed"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let user = canvas.get_user(UserId::Id(10)).await.unwrap();
    let channel = user
        .create_communication_channel("alice@newdomain.com", "email")
        .await
        .unwrap();

    assert_eq!(channel.id, 57);
    assert_eq!(channel.address.as_deref(), Some("alice@newdomain.com"));
    assert_eq!(channel.workflow_state.as_deref(), Some("unconfirmed"));
}
