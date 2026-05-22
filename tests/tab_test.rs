use canvas_lms_api::{resources::tab::UpdateTabParams, Canvas};
use futures::StreamExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn tab_json(id: &str, position: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "label": "Assignments",
        "type": "internal",
        "hidden": false,
        "position": position,
        "visibility": "public"
    })
}

#[tokio::test]
async fn test_course_tab_update() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/tabs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            tab_json("home", 1),
            tab_json("assignments", 2),
        ])))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/tabs/assignments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tab_json("assignments", 3)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();

    let tabs: Vec<_> = course
        .get_tabs()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(tabs.len(), 2);
    assert_eq!(tabs[1].course_id, Some(1));

    let updated = tabs[1]
        .update(UpdateTabParams {
            position: Some(3),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.position, Some(3));
    assert_eq!(updated.id.as_deref(), Some("assignments"));
}

#[tokio::test]
async fn test_group_tab_update_fails() {
    // A Tab without course_id (e.g. from a group) cannot be updated — returns error.
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/tabs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            tab_json("home", 1)
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let mut tabs: Vec<_> = course
        .get_tabs()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // Clear course_id to simulate a group tab
    tabs[0].course_id = None;
    let result = tabs[0]
        .update(UpdateTabParams {
            position: Some(1),
            ..Default::default()
        })
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        canvas_lms_api::CanvasError::BadRequest { .. }
    ));
}
