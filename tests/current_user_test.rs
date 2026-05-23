use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup(server: &MockServer) -> canvas_lms_api::resources::user::CurrentUser {
    Mock::given(method("GET"))
        .and(path("/api/v1/users/self"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "Self User",
            "login_id": "self@example.com"
        })))
        .mount(server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas.get_current_user().await.unwrap()
}

#[tokio::test]
async fn test_current_user_add_favorite_course() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/users/self/favorites/courses/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "context_id": 10,
            "context_type": "Course"
        })))
        .mount(&server)
        .await;

    let fav = user.add_favorite_course(10).await.unwrap();
    assert_eq!(fav.context_id, Some(10));
    assert_eq!(fav.context_type.as_deref(), Some("Course"));
}

#[tokio::test]
async fn test_current_user_add_favorite_group() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/users/self/favorites/groups/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "context_id": 5,
            "context_type": "Group"
        })))
        .mount(&server)
        .await;

    let fav = user.add_favorite_group(5).await.unwrap();
    assert_eq!(fav.context_id, Some(5));
    assert_eq!(fav.context_type.as_deref(), Some("Group"));
}

#[tokio::test]
async fn test_current_user_get_favorite_courses() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/self/favorites/courses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "Intro to Rust"},
            {"id": 2, "name": "Advanced Rust"}
        ])))
        .mount(&server)
        .await;

    let courses = user.get_favorite_courses().collect_all().await.unwrap();
    assert_eq!(courses.len(), 2);
    assert_eq!(courses[0].id, 1);
}

#[tokio::test]
async fn test_current_user_get_favorite_groups() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/self/favorites/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 3, "name": "Study Group A"},
            {"id": 4, "name": "Study Group B"}
        ])))
        .mount(&server)
        .await;

    let groups = user.get_favorite_groups().collect_all().await.unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].id, 3);
}

#[tokio::test]
async fn test_current_user_reset_favorite_courses() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/users/self/favorites/courses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    user.reset_favorite_courses().await.unwrap();
}

#[tokio::test]
async fn test_current_user_reset_favorite_groups() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/users/self/favorites/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    user.reset_favorite_groups().await.unwrap();
}

#[tokio::test]
async fn test_current_user_create_bookmark() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/users/self/bookmarks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "name": "My Bookmark",
            "url": "https://example.com/course/1",
            "position": 1
        })))
        .mount(&server)
        .await;

    let bm = user
        .create_bookmark("My Bookmark", "https://example.com/course/1")
        .await
        .unwrap();
    assert_eq!(bm.id, Some(7));
    assert_eq!(bm.name.as_deref(), Some("My Bookmark"));
}

#[tokio::test]
async fn test_current_user_get_bookmark() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/self/bookmarks/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "name": "My Bookmark",
            "url": "https://example.com/course/1"
        })))
        .mount(&server)
        .await;

    let bm = user.get_bookmark(7).await.unwrap();
    assert_eq!(bm.id, Some(7));
    assert_eq!(bm.url.as_deref(), Some("https://example.com/course/1"));
}

#[tokio::test]
async fn test_current_user_get_bookmarks() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/self/bookmarks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "First", "url": "https://example.com/1"},
            {"id": 2, "name": "Second", "url": "https://example.com/2"}
        ])))
        .mount(&server)
        .await;

    let bms = user.get_bookmarks().collect_all().await.unwrap();
    assert_eq!(bms.len(), 2);
    assert_eq!(bms[0].id, Some(1));
}

#[tokio::test]
async fn test_current_user_get_groups() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/self/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "name": "Biology Study Group"},
            {"id": 11, "name": "Calculus Study Group"}
        ])))
        .mount(&server)
        .await;

    let groups = user.get_groups().collect_all().await.unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].id, 10);
}

#[tokio::test]
async fn test_bookmark_edit() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/self/bookmarks/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
            "name": "Old Name",
            "url": "https://example.com/old"
        })))
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/users/self/bookmarks/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
            "name": "New Name",
            "url": "https://example.com/old"
        })))
        .mount(&server)
        .await;

    let bm = user.get_bookmark(3).await.unwrap();
    let updated = bm
        .edit(&[("name".to_string(), "New Name".to_string())])
        .await
        .unwrap();
    assert_eq!(updated.name.as_deref(), Some("New Name"));
}

#[tokio::test]
async fn test_bookmark_delete() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/self/bookmarks/4"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 4,
            "name": "To Delete",
            "url": "https://example.com/delete"
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/users/self/bookmarks/4"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    let bm = user.get_bookmark(4).await.unwrap();
    bm.delete().await.unwrap();
}
