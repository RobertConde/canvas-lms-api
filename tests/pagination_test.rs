#![cfg(feature = "async")]
use canvas_lms_api::Canvas;
use futures::StreamExt;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Mount a two-page courses mock. The first mock responds exactly once with a
/// Link header; the second mock handles the page-2 URL.
async fn two_page_mock(
    server: &MockServer,
    items_p1: serde_json::Value,
    items_p2: serde_json::Value,
) {
    let next_url = format!("{}/api/v1/courses?page=2&per_page=100", server.uri());

    // First page — only matches once so the second request falls through to page-2 mock.
    Mock::given(method("GET"))
        .and(path("/api/v1/courses"))
        .and(query_param("per_page", "100"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("Link", format!(r#"<{next_url}>; rel="next""#))
                .set_body_json(items_p1),
        )
        .up_to_n_times(1)
        .mount(server)
        .await;

    // Second page.
    Mock::given(method("GET"))
        .and(path("/api/v1/courses"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(items_p2))
        .mount(server)
        .await;
}

#[tokio::test]
async fn test_stream_collect_all_two_pages() {
    let server = MockServer::start().await;
    two_page_mock(
        &server,
        serde_json::json!([{"id": 1, "name": "Course 1"}, {"id": 2, "name": "Course 2"}]),
        serde_json::json!([{"id": 3, "name": "Course 3"}]),
    )
    .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let courses = canvas.get_courses().collect_all().await.unwrap();

    assert_eq!(courses.len(), 3);
    assert_eq!(courses[0].id, 1);
    assert_eq!(courses[2].id, 3);
}

#[tokio::test]
async fn test_stream_next_item_by_item() {
    let server = MockServer::start().await;
    two_page_mock(
        &server,
        serde_json::json!([{"id": 10, "name": "A"}, {"id": 20, "name": "B"}]),
        serde_json::json!([{"id": 30, "name": "C"}]),
    )
    .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let mut stream = canvas.get_courses();

    let c1 = stream.next().await.unwrap().unwrap();
    assert_eq!(c1.id, 10);
    let c2 = stream.next().await.unwrap().unwrap();
    assert_eq!(c2.id, 20);
    let c3 = stream.next().await.unwrap().unwrap();
    assert_eq!(c3.id, 30);
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn test_stream_map_via_streamext() {
    let server = MockServer::start().await;
    two_page_mock(
        &server,
        serde_json::json!([{"id": 1}, {"id": 2}]),
        serde_json::json!([{"id": 3}]),
    )
    .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let ids: Vec<u64> = canvas
        .get_courses()
        .map(|r| r.map(|c| c.id))
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<canvas_lms_api::Result<Vec<_>>>()
        .unwrap();

    assert_eq!(ids, vec![1, 2, 3]);
}

#[tokio::test]
async fn test_stream_error_propagates() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses"))
        .respond_with(
            ResponseTemplate::new(403).set_body_json(serde_json::json!({"status": "forbidden"})),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let result = canvas.get_courses().next().await.unwrap();
    assert!(matches!(
        result,
        Err(canvas_lms_api::CanvasError::Forbidden(_))
    ));
}
