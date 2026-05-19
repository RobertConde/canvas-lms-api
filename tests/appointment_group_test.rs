use canvas_lms_api::resources::appointment_group::AppointmentGroupParams;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn ag_json(id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "title": "Office Hours",
        "context_codes": ["course_1"],
        "workflow_state": "active",
        "appointments_count": 0
    })
}

#[tokio::test]
async fn test_get_appointment_group() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/appointment_groups/8"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ag_json(8)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let a = canvas.get_appointment_group(8).await.unwrap();
    assert_eq!(a.id, 8);
    assert_eq!(a.title.as_deref(), Some("Office Hours"));
}

#[tokio::test]
async fn test_get_appointment_groups() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/appointment_groups"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([ag_json(1), ag_json(2)])),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let groups = canvas.get_appointment_groups().collect_all().await.unwrap();
    assert_eq!(groups.len(), 2);
}

#[tokio::test]
async fn test_create_appointment_group() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/appointment_groups"))
        .respond_with(ResponseTemplate::new(201).set_body_json(ag_json(99)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let a = canvas
        .create_appointment_group(AppointmentGroupParams {
            context_codes: vec!["course_1".into()],
            title: Some("Office Hours".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(a.id, 99);
}

#[tokio::test]
async fn test_appointment_group_delete() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/appointment_groups/8"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ag_json(8)))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/appointment_groups/8"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ag_json(8)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let a = canvas.get_appointment_group(8).await.unwrap();
    let deleted = a.delete().await.unwrap();
    assert_eq!(deleted.id, 8);
}

#[tokio::test]
async fn test_appointment_group_edit() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/appointment_groups/8"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ag_json(8)))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/appointment_groups/8"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 8, "title": "Updated Hours", "context_codes": ["course_1"],
            "workflow_state": "active"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let a = canvas.get_appointment_group(8).await.unwrap();
    let updated = a
        .edit(AppointmentGroupParams {
            context_codes: vec!["course_1".into()],
            title: Some("Updated Hours".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.title.as_deref(), Some("Updated Hours"));
}
