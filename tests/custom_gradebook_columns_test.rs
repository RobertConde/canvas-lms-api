use canvas_lms_api::resources::custom_gradebook_column::CustomGradebookColumnParams;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn make_course(server: &MockServer) -> canvas_lms_api::resources::course::Course {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Canvas::new(&server.uri(), "test-token")
        .unwrap()
        .get_course(1)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_course_get_custom_columns() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/custom_gradebook_columns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "title": "Notes", "course_id": 1, "position": 1},
            {"id": 2, "title": "Extra Credit", "course_id": 1, "position": 2}
        ])))
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let columns = course.get_custom_columns().collect_all().await.unwrap();
    assert_eq!(columns.len(), 2);
    assert_eq!(columns[0].id, 1);
    assert_eq!(columns[0].title.as_deref(), Some("Notes"));
    assert_eq!(columns[1].id, 2);
    assert_eq!(columns[1].title.as_deref(), Some("Extra Credit"));
}

#[tokio::test]
async fn test_course_create_custom_column() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/custom_gradebook_columns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
            "title": "My Column",
            "course_id": 1,
            "position": 1
        })))
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let col = course
        .create_custom_column(CustomGradebookColumnParams {
            title: Some("My Column".to_string()),
            position: Some(1),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(col.id, 3);
    assert_eq!(col.title.as_deref(), Some("My Column"));
    assert_eq!(col.course_id, Some(1));
}

#[tokio::test]
async fn test_column_update() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/custom_gradebook_columns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "title": "Notes", "course_id": 1, "position": 1}
        ])))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/custom_gradebook_columns/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "title": "Updated Notes",
            "course_id": 1,
            "position": 1
        })))
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let columns = course.get_custom_columns().collect_all().await.unwrap();
    let col = &columns[0];

    let updated = col
        .update(CustomGradebookColumnParams {
            title: Some("Updated Notes".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.id, 1);
    assert_eq!(updated.title.as_deref(), Some("Updated Notes"));
}

#[tokio::test]
async fn test_column_delete() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/custom_gradebook_columns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "title": "Notes", "course_id": 1, "position": 1}
        ])))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/custom_gradebook_columns/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "title": "Notes",
            "course_id": 1,
            "position": 1
        })))
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let columns = course.get_custom_columns().collect_all().await.unwrap();
    let col = &columns[0];

    let deleted = col.delete().await.unwrap();
    assert_eq!(deleted.id, 1);
}

#[tokio::test]
async fn test_column_get_entries() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/custom_gradebook_columns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "title": "Notes", "course_id": 1, "position": 1}
        ])))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/custom_gradebook_columns/1/data"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "gradebook_column_id": 1, "content": "Good work", "user_id": 100},
            {"id": 11, "gradebook_column_id": 1, "content": "Needs improvement", "user_id": 101}
        ])))
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let columns = course.get_custom_columns().collect_all().await.unwrap();
    let col = &columns[0];

    let entries = col.get_column_entries().collect_all().await.unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].content.as_deref(), Some("Good work"));
    assert_eq!(entries[0].user_id, Some(100));
    assert_eq!(entries[1].content.as_deref(), Some("Needs improvement"));
}

#[tokio::test]
async fn test_column_data_update() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/custom_gradebook_columns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "title": "Notes", "course_id": 1, "position": 1}
        ])))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/custom_gradebook_columns/1/data"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "gradebook_column_id": 1, "content": "Old content", "user_id": 100}
        ])))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path(
            "/api/v1/courses/1/custom_gradebook_columns/1/data/100",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "gradebook_column_id": 1,
            "content": "New content",
            "user_id": 100
        })))
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let columns = course.get_custom_columns().collect_all().await.unwrap();
    let col = &columns[0];

    let entries = col.get_column_entries().collect_all().await.unwrap();
    let entry = &entries[0];

    let updated = entry.update_column_data("New content").await.unwrap();
    assert_eq!(updated.content.as_deref(), Some("New content"));
    assert_eq!(updated.user_id, Some(100));
}

#[tokio::test]
async fn test_reorder_custom_columns() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/custom_gradebook_columns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "title": "Notes", "course_id": 1, "position": 1}
        ])))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/custom_gradebook_columns/reorder"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let columns = course.get_custom_columns().collect_all().await.unwrap();
    let col = &columns[0];
    col.reorder_custom_columns(&[2, 1]).await.unwrap();
}
