use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn make_account(server: &MockServer) -> canvas_lms_api::resources::account::Account {
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Canvas::new(&server.uri(), "token")
        .unwrap()
        .get_account(1)
        .await
        .unwrap()
}

fn login_json(id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "account_id": 1,
        "user_id": 10,
        "unique_id": "user@example.com",
        "workflow_state": "active"
    })
}

#[tokio::test]
async fn test_account_get_user_logins() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/logins"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([login_json(101), login_json(102)])),
        )
        .mount(&server)
        .await;

    let logins = account.get_user_logins().collect_all().await.unwrap();
    assert_eq!(logins.len(), 2);
    assert_eq!(logins[0].id, 101);
    assert_eq!(logins[0].account_id, Some(1));
    assert_eq!(logins[0].unique_id.as_deref(), Some("user@example.com"));
}

#[tokio::test]
async fn test_account_create_user_login() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/logins"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 101,
            "account_id": 1,
            "user_id": 1,
            "unique_id": "belieber@example.com",
            "workflow_state": "active"
        })))
        .mount(&server)
        .await;

    let login = account
        .create_user_login(&[
            ("user[id]".to_string(), "1".to_string()),
            (
                "login[unique_id]".to_string(),
                "belieber@example.com".to_string(),
            ),
        ])
        .await
        .unwrap();
    assert_eq!(login.id, 101);
    assert_eq!(login.unique_id.as_deref(), Some("belieber@example.com"));
    assert_eq!(login.account_id, Some(1));
}

#[tokio::test]
async fn test_login_edit() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/logins"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([login_json(101)])),
        )
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/accounts/1/logins/101"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 101,
            "account_id": 1,
            "user_id": 10,
            "unique_id": "newemail@example.com",
            "workflow_state": "active"
        })))
        .mount(&server)
        .await;

    let login = account
        .get_user_logins()
        .collect_all()
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    let updated = login
        .edit(&[(
            "login[unique_id]".to_string(),
            "newemail@example.com".to_string(),
        )])
        .await
        .unwrap();
    assert_eq!(updated.unique_id.as_deref(), Some("newemail@example.com"));
}

#[tokio::test]
async fn test_login_delete() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/logins"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([login_json(101)])),
        )
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/users/10/logins/101"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 101,
            "account_id": 1,
            "user_id": 10,
            "unique_id": "user@example.com",
            "workflow_state": "deleted"
        })))
        .mount(&server)
        .await;

    let login = account
        .get_user_logins()
        .collect_all()
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    let deleted = login.delete().await.unwrap();
    assert_eq!(deleted.workflow_state.as_deref(), Some("deleted"));
}

#[tokio::test]
async fn test_login_get_authentication_events() {
    let server = MockServer::start().await;
    let account = make_account(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/logins"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([login_json(101)])),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/audit/authentication/logins/101"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"event_type": "login", "pseudonym_id": 9478, "created_at": "2012-07-19T15:00:00-06:00"},
            {"event_type": "logout", "pseudonym_id": 9478, "created_at": "2012-07-20T15:00:00-06:00"}
        ])))
        .mount(&server)
        .await;

    let login = account
        .get_user_logins()
        .collect_all()
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    let events = login
        .get_authentication_events()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_type.as_deref(), Some("login"));
    assert_eq!(events[1].event_type.as_deref(), Some("logout"));
}
