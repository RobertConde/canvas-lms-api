use canvas_lms_api::{resources::page::UpdatePageParams, Canvas};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn page_json(url: &str, title: &str) -> serde_json::Value {
    serde_json::json!({
        "url": url,
        "title": title,
        "body": "<p>Hello</p>",
        "published": true
    })
}

fn revision_json(revision_id: u64) -> serde_json::Value {
    serde_json::json!({
        "revision_id": revision_id,
        "url": "welcome",
        "title": "Welcome",
        "latest": false,
        "body": "<p>v1</p>"
    })
}

async fn setup_course_page(server: &MockServer) -> canvas_lms_api::resources::page::Page {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/pages/welcome"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_json("welcome", "Welcome")))
        .mount(server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    course.get_page("welcome").await.unwrap()
}

#[tokio::test]
async fn test_page_edit() {
    let server = MockServer::start().await;
    let page = setup_course_page(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/pages/welcome"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_json("welcome", "Welcome Updated")),
        )
        .mount(&server)
        .await;

    let updated = page
        .edit(UpdatePageParams {
            title: Some("Welcome Updated".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.title.as_deref(), Some("Welcome Updated"));
    assert_eq!(updated.course_id, Some(1));
}

#[tokio::test]
async fn test_page_delete_course() {
    let server = MockServer::start().await;
    let page = setup_course_page(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/pages/welcome"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_json("welcome", "Welcome")))
        .mount(&server)
        .await;

    let deleted = page.delete().await.unwrap();
    assert_eq!(deleted.url.as_deref(), Some("welcome"));
}

#[tokio::test]
async fn test_page_get_revisions() {
    let server = MockServer::start().await;
    let page = setup_course_page(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/pages/welcome/revisions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([revision_json(1), revision_json(2)])),
        )
        .mount(&server)
        .await;

    let revisions: Vec<_> = page
        .get_revisions()
        .unwrap()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(revisions.len(), 2);
    assert_eq!(revisions[0].revision_id, Some(1));
}

#[tokio::test]
async fn test_page_show_latest_revision() {
    let server = MockServer::start().await;
    let page = setup_course_page(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/pages/welcome/revisions/latest"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "revision_id": 5,
                "latest": true,
                "url": "welcome",
                "title": "Welcome"
            })),
        )
        .mount(&server)
        .await;

    let rev = page.show_latest_revision().await.unwrap();
    assert_eq!(rev.revision_id, Some(5));
    assert_eq!(rev.latest, Some(true));
}

#[tokio::test]
async fn test_page_get_revision_by_id() {
    let server = MockServer::start().await;
    let page = setup_course_page(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/pages/welcome/revisions/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(revision_json(3)))
        .mount(&server)
        .await;

    let rev = page.get_revision_by_id(3).await.unwrap();
    assert_eq!(rev.revision_id, Some(3));
    assert_eq!(rev.course_id, Some(1));
}

#[tokio::test]
async fn test_page_revert_to_revision() {
    let server = MockServer::start().await;
    let page = setup_course_page(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/pages/welcome/revisions/2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(revision_json(2)))
        .mount(&server)
        .await;

    let rev = page.revert_to_revision(2).await.unwrap();
    assert_eq!(rev.revision_id, Some(2));
}

#[tokio::test]
async fn test_page_delete_group() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/pages/welcome"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_json("welcome", "Welcome")))
        .mount(&server)
        .await;

    // Simulate group page by overriding group_id after fetch
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let mut page = course.get_page("welcome").await.unwrap();

    // Switch to group context
    page.course_id = None;
    page.group_id = Some(5);

    Mock::given(method("DELETE"))
        .and(path("/api/v1/groups/5/pages/welcome"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_json("welcome", "Welcome")))
        .mount(&server)
        .await;

    let deleted = page.delete().await.unwrap();
    assert_eq!(deleted.url.as_deref(), Some("welcome"));
}
