use canvas_lms_api::resources::planner::{PlannerNoteParams, PlannerOverrideParams};
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn note_json(id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "title": "Study for exam",
        "todo_date": "2026-06-10T00:00:00Z",
        "workflow_state": "active"
    })
}

fn override_json(id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "plannable_type": "assignment",
        "plannable_id": 55,
        "marked_complete": false,
        "dismissed": false
    })
}

#[tokio::test]
async fn test_get_planner_note() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/planner_notes/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(note_json(3)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let n = canvas.get_planner_note(3).await.unwrap();
    assert_eq!(n.id, 3);
    assert_eq!(n.title.as_deref(), Some("Study for exam"));
}

#[tokio::test]
async fn test_get_planner_notes() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/planner_notes"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([note_json(1), note_json(2)])),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let notes = canvas.get_planner_notes().collect_all().await.unwrap();
    assert_eq!(notes.len(), 2);
}

#[tokio::test]
async fn test_create_planner_note() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/planner_notes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(note_json(10)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let n = canvas
        .create_planner_note(PlannerNoteParams {
            title: Some("Study for exam".into()),
            todo_date: Some("2026-06-10T00:00:00Z".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(n.id, 10);
}

#[tokio::test]
async fn test_planner_note_delete() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/planner_notes/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(note_json(3)))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/planner_notes/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(note_json(3)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let n = canvas.get_planner_note(3).await.unwrap();
    let deleted = n.delete().await.unwrap();
    assert_eq!(deleted.id, 3);
}

#[tokio::test]
async fn test_planner_note_update() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/planner_notes/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(note_json(3)))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/planner_notes/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3, "title": "Updated Note", "workflow_state": "active"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let n = canvas.get_planner_note(3).await.unwrap();
    let updated = n
        .update(PlannerNoteParams {
            title: Some("Updated Note".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.title.as_deref(), Some("Updated Note"));
}

#[tokio::test]
async fn test_get_planner_override() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/planner/overrides/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(override_json(7)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let o = canvas.get_planner_override(7).await.unwrap();
    assert_eq!(o.id, 7);
    assert_eq!(o.plannable_type.as_deref(), Some("assignment"));
}

#[tokio::test]
async fn test_create_planner_override() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/planner/overrides"))
        .respond_with(ResponseTemplate::new(200).set_body_json(override_json(20)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let o = canvas
        .create_planner_override("assignment", 55)
        .await
        .unwrap();
    assert_eq!(o.id, 20);
}

#[tokio::test]
async fn test_planner_override_update() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/planner/overrides/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(override_json(7)))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/planner/overrides/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7, "plannable_type": "assignment", "plannable_id": 55,
            "marked_complete": true, "dismissed": false
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let o = canvas.get_planner_override(7).await.unwrap();
    let updated = o
        .update(PlannerOverrideParams {
            marked_complete: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.marked_complete, Some(true));
}

#[tokio::test]
async fn test_get_planner_overrides() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/planner/overrides"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([override_json(1), override_json(2)])),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let overrides = canvas.get_planner_overrides().collect_all().await.unwrap();
    assert_eq!(overrides.len(), 2);
}

#[tokio::test]
async fn test_planner_override_delete() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/planner/overrides/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(override_json(7)))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/planner/overrides/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(override_json(7)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let o = canvas.get_planner_override(7).await.unwrap();
    let deleted = o.delete().await.unwrap();
    assert_eq!(deleted.id, 7);
}
