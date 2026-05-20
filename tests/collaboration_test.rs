use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn collab_json(id: u64, context_type: &str, context_id: u64) -> serde_json::Value {
    serde_json::json!([{
        "id": id,
        "collaboration_type": "GoogleDocs",
        "document_id": "doc123",
        "context_type": context_type,
        "context_id": context_id,
        "title": "Shared Doc"
    }])
}

#[tokio::test]
async fn test_course_get_collaborations() {
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
        .and(path("/api/v1/courses/1/collaborations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(collab_json(10, "Course", 1)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let collabs = course.get_collaborations().collect_all().await.unwrap();
    assert_eq!(collabs.len(), 1);
    assert_eq!(collabs[0].id, 10);
    assert_eq!(collabs[0].title.as_deref(), Some("Shared Doc"));
}

#[tokio::test]
async fn test_group_get_collaborations() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/groups/5"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"id": 5, "name": "Test Group"})),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/groups/5/collaborations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(collab_json(20, "Group", 5)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let group = canvas.get_group(5).await.unwrap();
    let collabs = group.get_collaborations().collect_all().await.unwrap();
    assert_eq!(collabs.len(), 1);
    assert_eq!(collabs[0].id, 20);
    assert_eq!(collabs[0].collaboration_type.as_deref(), Some("GoogleDocs"));
}

#[tokio::test]
async fn test_collaboration_get_collaborators() {
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
        .and(path("/api/v1/courses/1/collaborations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(collab_json(10, "Course", 1)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/collaborations/10/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 100, "type": "user", "name": "Alice"},
            {"id": 101, "type": "user", "name": "Bob"}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let collabs = course.get_collaborations().collect_all().await.unwrap();
    let collaborators = collabs[0].get_collaborators().collect_all().await.unwrap();
    assert_eq!(collaborators.len(), 2);
    assert_eq!(collaborators[0].name.as_deref(), Some("Alice"));
}
