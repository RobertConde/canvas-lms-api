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

async fn make_channel(
    server: &MockServer,
) -> canvas_lms_api::resources::communication_channel::CommunicationChannel {
    Mock::given(method("GET"))
        .and(path("/api/v1/users/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 10})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/users/10/communication_channels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 55, "address": "alice@example.com", "type": "email", "user_id": 10, "workflow_state": "active"}
        ])))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas
        .get_user(UserId::Id(10))
        .await
        .unwrap()
        .get_communication_channels()
        .collect_all()
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
}

#[tokio::test]
async fn test_channel_get_preference() {
    let server = MockServer::start().await;
    let channel = make_channel(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/users/10/communication_channels/55/notification_preferences/Assignment%20Created",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "notification_preferences": [
                {"notification": "Assignment Created", "frequency": "immediately"}
            ]
        })))
        .mount(&server)
        .await;

    let pref = channel.get_preference("Assignment Created").await.unwrap();
    assert_eq!(pref["frequency"], "immediately");
}

#[tokio::test]
async fn test_channel_get_preference_categories() {
    let server = MockServer::start().await;
    let channel = make_channel(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/users/10/communication_channels/55/notification_preference_categories",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "categories": ["due_date", "grading", "invitation"]
        })))
        .mount(&server)
        .await;

    let cats = channel.get_preference_categories().await.unwrap();
    assert_eq!(cats.len(), 3);
    assert!(cats.contains(&"due_date".to_string()));
}

#[tokio::test]
async fn test_channel_update_preference() {
    let server = MockServer::start().await;
    let channel = make_channel(&server).await;

    Mock::given(method("PUT"))
        .and(path(
            "/api/v1/users/self/communication_channels/55/notification_preferences/Assignment%20Due%20Date",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "notification_preferences": [
                {"notification": "Assignment Due Date", "frequency": "daily"}
            ]
        })))
        .mount(&server)
        .await;

    let updated = channel
        .update_preference("Assignment Due Date", "daily")
        .await
        .unwrap();
    assert_eq!(updated["frequency"], "daily");
}

#[tokio::test]
async fn test_channel_update_preferences_by_category() {
    let server = MockServer::start().await;
    let channel = make_channel(&server).await;

    Mock::given(method("PUT"))
        .and(path(
            "/api/v1/users/self/communication_channels/55/notification_preference_categories/grading",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "notification_preferences": [
                {"notification": "Grade Posted", "frequency": "never"},
                {"notification": "Submission Graded", "frequency": "never"}
            ]
        })))
        .mount(&server)
        .await;

    let prefs = channel
        .update_preferences_by_category("grading", "never")
        .await
        .unwrap();
    assert_eq!(prefs.len(), 2);
    assert_eq!(prefs[0]["frequency"], "never");
}
