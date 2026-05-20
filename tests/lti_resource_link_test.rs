use canvas_lms_api::Canvas;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn link_json(id: u64) -> serde_json::Value {
    serde_json::json!({"id": id, "url": "https://tool.example/launch", "title": "My LTI Tool"})
}

#[tokio::test]
async fn test_get_lti_resource_links() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"id": 1, "name": "Test Course"})),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/lti_resource_links"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([link_json(10), link_json(11)])),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let links = course.get_lti_resource_links().collect_all().await.unwrap();
    assert_eq!(links.len(), 2);
    assert_eq!(links[0].id, 10);
    assert_eq!(links[0].url.as_deref(), Some("https://tool.example/launch"));
}

#[tokio::test]
async fn test_get_lti_resource_link() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"id": 1, "name": "Test Course"})),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/lti_resource_links/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(link_json(10)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let link = course.get_lti_resource_link(10).await.unwrap();
    assert_eq!(link.id, 10);
    assert_eq!(link.title.as_deref(), Some("My LTI Tool"));
}

#[tokio::test]
async fn test_create_lti_resource_link() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"id": 1, "name": "Test Course"})),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/lti_resource_links"))
        .and(body_string_contains("url=https"))
        .respond_with(ResponseTemplate::new(200).set_body_json(link_json(20)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let link = course
        .create_lti_resource_link(
            canvas_lms_api::resources::lti_resource_link::CreateLtiResourceLinkParams {
                url: "https://tool.example/launch".into(),
                title: Some("My LTI Tool".into()),
                custom: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(link.id, 20);
    assert_eq!(link.title.as_deref(), Some("My LTI Tool"));
}
