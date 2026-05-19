use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn event_json(id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "event_type": "grade_change",
        "grade_before": "B",
        "grade_after": "A",
        "grade_current": "A",
        "course_id": 1,
        "assignment_id": 42,
        "student_id": 10,
        "grader_id": 5,
        "created_at": "2026-05-01T12:00:00Z"
    })
}

async fn make_course(server: &MockServer) -> canvas_lms_api::resources::course::Course {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Canvas::new(&server.uri(), "token")
        .unwrap()
        .get_course(1)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_get_grade_change_events() {
    let server = MockServer::start().await;
    // Canvas wraps grade change events in {"events": [...]}
    Mock::given(method("GET"))
        .and(path("/api/v1/audit/grade_change/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "events": [event_json("evt-1"), event_json("evt-2")]
        })))
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let events = course
        .get_grade_change_events()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_type.as_deref(), Some("grade_change"));
    assert_eq!(events[0].grade_before.as_deref(), Some("B"));
    assert_eq!(events[0].grade_after.as_deref(), Some("A"));
    assert_eq!(events[1].id.as_deref(), Some("evt-2"));
}

#[tokio::test]
async fn test_get_grade_change_events_empty() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/audit/grade_change/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "events": [] })))
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let events = course
        .get_grade_change_events()
        .collect_all()
        .await
        .unwrap();
    assert!(events.is_empty());
}
