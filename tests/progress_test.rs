use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn progress_json(id: u64, state: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "tag": "course_export",
        "completion": 50.0,
        "workflow_state": state,
        "message": null
    })
}

#[tokio::test]
async fn test_progress_query() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/progress/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(progress_json(1, "running")))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    // Second GET for query()
    Mock::given(method("GET"))
        .and(path("/api/v1/progress/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(progress_json(1, "completed")))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let progress = canvas.get_progress(1).await.unwrap();
    assert_eq!(progress.id, 1);
    assert_eq!(progress.workflow_state.as_deref(), Some("running"));

    let updated = progress.query().await.unwrap();
    assert_eq!(updated.workflow_state.as_deref(), Some("completed"));
}
