use canvas_lms_api::resources::calendar_event::CalendarEventParams;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn event_json(id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "title": "Team Meeting",
        "start_at": "2026-06-01T10:00:00Z",
        "end_at": "2026-06-01T11:00:00Z",
        "context_code": "course_1",
        "workflow_state": "active"
    })
}

#[tokio::test]
async fn test_get_calendar_event() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/calendar_events/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(event_json(5)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let e = canvas.get_calendar_event(5).await.unwrap();
    assert_eq!(e.id, 5);
    assert_eq!(e.title.as_deref(), Some("Team Meeting"));
}

#[tokio::test]
async fn test_get_calendar_events() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/calendar_events"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([event_json(1), event_json(2)])),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let events = canvas.get_calendar_events().collect_all().await.unwrap();
    assert_eq!(events.len(), 2);
}

#[tokio::test]
async fn test_create_calendar_event() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/calendar_events"))
        .respond_with(ResponseTemplate::new(201).set_body_json(event_json(99)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let e = canvas
        .create_calendar_event(
            "course_1",
            CalendarEventParams {
                title: Some("Team Meeting".into()),
                start_at: Some("2026-06-01T10:00:00Z".into()),
                end_at: Some("2026-06-01T11:00:00Z".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(e.id, 99);
}

#[tokio::test]
async fn test_calendar_event_delete() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/calendar_events/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(event_json(5)))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/calendar_events/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(event_json(5)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let e = canvas.get_calendar_event(5).await.unwrap();
    let deleted = e.delete().await.unwrap();
    assert_eq!(deleted.id, 5);
}

#[tokio::test]
async fn test_calendar_event_edit() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/calendar_events/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(event_json(5)))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/calendar_events/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5, "title": "Updated Meeting", "workflow_state": "active"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let e = canvas.get_calendar_event(5).await.unwrap();
    let updated = e
        .edit(CalendarEventParams {
            title: Some("Updated Meeting".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.title.as_deref(), Some("Updated Meeting"));
}
