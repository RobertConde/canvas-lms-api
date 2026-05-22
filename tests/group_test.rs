use canvas_lms_api::{
    resources::group::{GroupCategoryParams, UpdateGroupParams, UpdateMembershipParams},
    Canvas,
};
use futures::StreamExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn group_json(id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": "Group 1",
        "members_count": 3
    })
}

fn membership_json(id: u64, group_id: u64, user_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "group_id": group_id,
        "user_id": user_id,
        "workflow_state": "accepted",
        "moderator": false
    })
}

fn group_category_json(id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": "Project Groups",
        "self_signup": "enabled",
        "groups_count": 2
    })
}

async fn setup(server: &MockServer) -> canvas_lms_api::resources::group::Group {
    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(group_json(1)))
        .mount(server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas.get_group(1).await.unwrap()
}

#[tokio::test]
async fn test_group_edit() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/groups/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "Updated Group"
        })))
        .mount(&server)
        .await;

    let updated = group
        .edit(UpdateGroupParams {
            name: Some("Updated Group".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.name.as_deref(), Some("Updated Group"));
}

#[tokio::test]
async fn test_group_delete() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/groups/1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    group.delete().await.unwrap();
}

#[tokio::test]
async fn test_group_get_users() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "name": "Alice"},
            {"id": 11, "name": "Bob"}
        ])))
        .mount(&server)
        .await;

    let users: Vec<_> = group
        .get_users()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].id, 10);
}

#[tokio::test]
async fn test_group_get_memberships() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/memberships"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            membership_json(20, 1, 10),
            membership_json(21, 1, 11)
        ])))
        .mount(&server)
        .await;

    let memberships: Vec<_> = group
        .get_memberships()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(memberships.len(), 2);
    assert_eq!(memberships[0].id, 20);
    assert_eq!(memberships[0].group_id, Some(1));
}

#[tokio::test]
async fn test_group_create_membership() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/groups/1/memberships"))
        .respond_with(ResponseTemplate::new(200).set_body_json(membership_json(22, 1, 10)))
        .mount(&server)
        .await;

    let m = group.create_membership(10).await.unwrap();
    assert_eq!(m.id, 22);
    assert_eq!(m.group_id, Some(1));
    assert_eq!(m.user_id, Some(10));
}

#[tokio::test]
async fn test_group_get_membership() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/users/10/membership"))
        .respond_with(ResponseTemplate::new(200).set_body_json(membership_json(20, 1, 10)))
        .mount(&server)
        .await;

    let m = group.get_membership(10).await.unwrap();
    assert_eq!(m.user_id, Some(10));
    assert_eq!(m.group_id, Some(1));
}

#[tokio::test]
async fn test_group_update_membership() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/groups/1/memberships/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20,
            "group_id": 1,
            "user_id": 10,
            "moderator": true
        })))
        .mount(&server)
        .await;

    let m = group
        .update_membership(
            20,
            UpdateMembershipParams {
                moderator: Some(true),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(m.moderator, Some(true));
}

#[tokio::test]
async fn test_group_remove_user() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/groups/1/users/10"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    group.remove_user(10).await.unwrap();
}

#[tokio::test]
async fn test_group_invite() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/groups/1/invite"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            membership_json(23, 1, 10),
            membership_json(24, 1, 11)
        ])))
        .mount(&server)
        .await;

    let memberships = group.invite(&[10, 11]).await.unwrap();
    assert_eq!(memberships.len(), 2);
    assert_eq!(memberships[0].group_id, Some(1));
}

#[tokio::test]
async fn test_group_get_files() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 100, "display_name": "file.pdf", "size": 1024,
             "url": "https://example.com/files/100", "content-type": "application/pdf"}
        ])))
        .mount(&server)
        .await;

    let files: Vec<_> = group
        .get_files()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].id, 100);
}

#[tokio::test]
async fn test_group_get_folders() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/folders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 200, "name": "Shared Files", "full_name": "Shared Files"}
        ])))
        .mount(&server)
        .await;

    let folders: Vec<_> = group
        .get_folders()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(folders.len(), 1);
    assert_eq!(folders[0].id, 200);
}

#[tokio::test]
async fn test_group_create_folder() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/groups/1/folders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 201,
            "name": "New Folder",
            "full_name": "New Folder"
        })))
        .mount(&server)
        .await;

    let folder = group.create_folder("New Folder").await.unwrap();
    assert_eq!(folder.id, 201);
}

#[tokio::test]
async fn test_group_get_pages() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/pages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"url": "welcome", "title": "Welcome", "body": "<p>Hello</p>", "published": true}
        ])))
        .mount(&server)
        .await;

    let pages: Vec<_> = group
        .get_pages()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].url.as_deref(), Some("welcome"));
    assert_eq!(pages[0].group_id, Some(1));
}

#[tokio::test]
async fn test_group_get_page() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/pages/welcome"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "url": "welcome",
            "title": "Welcome",
            "body": "<p>Hello</p>",
            "published": true
        })))
        .mount(&server)
        .await;

    let page = group.get_page("welcome").await.unwrap();
    assert_eq!(page.url.as_deref(), Some("welcome"));
    assert_eq!(page.group_id, Some(1));
}

#[tokio::test]
async fn test_group_get_discussion_topics() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/discussion_topics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 5, "title": "Discussion 1", "published": true}
        ])))
        .mount(&server)
        .await;

    let topics: Vec<_> = group
        .get_discussion_topics()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(topics.len(), 1);
    assert_eq!(topics[0].id, 5);
    assert_eq!(topics[0].group_id, Some(1));
}

#[tokio::test]
async fn test_group_get_discussion_topic() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/discussion_topics/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "title": "Discussion 1"
        })))
        .mount(&server)
        .await;

    let topic = group.get_discussion_topic(5).await.unwrap();
    assert_eq!(topic.id, 5);
    assert_eq!(topic.group_id, Some(1));
}

// GroupMembership methods

#[tokio::test]
async fn test_group_membership_update() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/memberships"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([membership_json(20, 1, 10)])),
        )
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/groups/1/memberships/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20,
            "group_id": 1,
            "user_id": 10,
            "moderator": true
        })))
        .mount(&server)
        .await;

    let memberships: Vec<_> = group
        .get_memberships()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    let m = &memberships[0];
    let updated = m
        .update(UpdateMembershipParams {
            moderator: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.moderator, Some(true));
}

#[tokio::test]
async fn test_group_membership_remove_self() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/memberships"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([membership_json(20, 1, 10)])),
        )
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/groups/1/memberships/20"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let memberships: Vec<_> = group
        .get_memberships()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    memberships[0].remove_self().await.unwrap();
}

// GroupCategory tests

#[tokio::test]
async fn test_course_get_group_categories() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/group_categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            group_category_json(10),
            group_category_json(11)
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let categories: Vec<_> = course
        .get_group_categories()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(categories.len(), 2);
    assert_eq!(categories[0].id, 10);
}

#[tokio::test]
async fn test_course_create_group_category() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/group_categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(group_category_json(12)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let gc = course
        .create_group_category(GroupCategoryParams {
            name: Some("Project Groups".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(gc.id, 12);
}

#[tokio::test]
async fn test_group_category_update() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/group_categories"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([group_category_json(10)])),
        )
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/group_categories/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "name": "Updated Category"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let categories: Vec<_> = course
        .get_group_categories()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    let updated = categories[0]
        .update(GroupCategoryParams {
            name: Some("Updated Category".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.name.as_deref(), Some("Updated Category"));
}

#[tokio::test]
async fn test_group_category_delete() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/group_categories"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([group_category_json(10)])),
        )
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/group_categories/10"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let categories: Vec<_> = course
        .get_group_categories()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    categories[0].delete().await.unwrap();
}

#[tokio::test]
async fn test_group_category_get_groups() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/group_categories"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([group_category_json(10)])),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/group_categories/10/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            group_json(1),
            group_json(2)
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let categories: Vec<_> = course
        .get_group_categories()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    let groups: Vec<_> = categories[0]
        .get_groups()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(groups.len(), 2);
}

#[tokio::test]
async fn test_group_create_page() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/groups/1/pages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "url": "new-page",
            "title": "New Page",
            "body": "Content"
        })))
        .mount(&server)
        .await;

    let page = group
        .create_page(&[
            ("wiki_page[title]".to_string(), "New Page".to_string()),
        ])
        .await
        .unwrap();
    assert_eq!(page.title.as_deref(), Some("New Page"));
    assert_eq!(page.group_id, Some(1));
}

#[tokio::test]
async fn test_group_create_discussion_topic() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/groups/1/discussion_topics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "title": "New Discussion",
            "group_id": 1
        })))
        .mount(&server)
        .await;

    let topic = group
        .create_discussion_topic(&[
            ("title".to_string(), "New Discussion".to_string()),
        ])
        .await
        .unwrap();
    assert_eq!(topic.id, 10);
    assert_eq!(topic.group_id, Some(1));
}

#[tokio::test]
async fn test_group_get_tabs() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/tabs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "home", "label": "Home", "type": "internal"},
            {"id": "files", "label": "Files", "type": "internal"}
        ])))
        .mount(&server)
        .await;

    let tabs: Vec<_> = group
        .get_tabs()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(tabs.len(), 2);
    assert_eq!(tabs[0]["id"], "home");
}

#[tokio::test]
async fn test_group_get_content_migrations() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/content_migrations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "migration_type": "common_cartridge_importer", "workflow_state": "completed"}
        ])))
        .mount(&server)
        .await;

    let migrations: Vec<_> = group
        .get_content_migrations()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(migrations.len(), 1);
    assert_eq!(migrations[0]["migration_type"], "common_cartridge_importer");
}

#[tokio::test]
async fn test_group_get_content_exports() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/content_exports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "export_type": "common_cartridge", "workflow_state": "exported"}
        ])))
        .mount(&server)
        .await;

    let exports: Vec<_> = group
        .get_content_exports()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(exports.len(), 1);
    assert_eq!(exports[0]["export_type"], "common_cartridge");
}

#[tokio::test]
async fn test_group_preview_html() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/groups/1/preview_html"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "html": "<p>Hello world</p>"
        })))
        .mount(&server)
        .await;

    let result = group.preview_html("<p>Hello world</p>").await.unwrap();
    assert_eq!(result["html"], "<p>Hello world</p>");
}

#[tokio::test]
async fn test_group_resolve_path() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/folders/by_path"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "course files", "full_name": "course files"}
        ])))
        .mount(&server)
        .await;

    let folders: Vec<_> = group
        .resolve_path(None)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(folders.len(), 1);
    assert_eq!(folders[0].name.as_deref(), Some("course files"));
}

#[tokio::test]
async fn test_group_category_get_users() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/group_categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "name": "Project Groups"}
        ])))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/group_categories/10/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 42, "name": "Alice"},
            {"id": 43, "name": "Bob"}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let categories: Vec<_> = course
        .get_group_categories()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    let users: Vec<_> = categories[0]
        .get_users()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].id, 42);
}

#[tokio::test]
async fn test_group_category_create_group() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/group_categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "name": "Project Groups"}
        ])))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/group_categories/10/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(
            {"id": 20, "name": "New Group", "members_count": 0}
        )))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let categories: Vec<_> = course
        .get_group_categories()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    let group = categories[0].create_group("New Group").await.unwrap();
    assert_eq!(group.id, 20);
}

#[tokio::test]
async fn test_group_category_assign_members() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/group_categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "name": "Project Groups"}
        ])))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/group_categories/10/assign_unassigned_members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "workflow_state": "queued",
            "completion": 0
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let categories: Vec<_> = course
        .get_group_categories()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    let progress = categories[0].assign_members().await.unwrap();
    assert_eq!(progress.id, 5);
    assert_eq!(progress.workflow_state.as_deref(), Some("queued"));
}
