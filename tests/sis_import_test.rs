use canvas_lms_api::resources::sis_import::SisImport;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_sis_import(server: &MockServer) -> SisImport {
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/sis_imports/2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 2,
            "account_id": 1,
            "workflow_state": "imported"
        })))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas
        .get_account(1)
        .await
        .unwrap()
        .get_sis_import(2)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_sis_import_abort() {
    let server = MockServer::start().await;
    let import = setup_sis_import(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/accounts/1/sis_imports/2/abort"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 2,
            "account_id": 1,
            "workflow_state": "aborted"
        })))
        .mount(&server)
        .await;

    let aborted = import.abort().await.unwrap();
    assert_eq!(aborted.id, 2);
    assert_eq!(aborted.workflow_state.as_deref(), Some("aborted"));
}

#[tokio::test]
async fn test_sis_import_restore_states() {
    let server = MockServer::start().await;
    let import = setup_sis_import(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/accounts/1/sis_imports/2/restore_states"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99,
            "workflow_state": "queued",
            "completion": 0
        })))
        .mount(&server)
        .await;

    let progress = import.restore_states().await.unwrap();
    assert_eq!(progress.id, 99);
    assert_eq!(progress.workflow_state.as_deref(), Some("queued"));
}
