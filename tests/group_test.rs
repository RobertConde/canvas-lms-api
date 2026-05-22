use canvas_lms_api::{
    resources::group::{GroupCategoryParams, UpdateGroupParams, UpdateMembershipParams},
    Canvas,
};
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

    let users: Vec<_> = group.get_users().collect_all().await.unwrap();
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

    let memberships: Vec<_> = group.get_memberships().collect_all().await.unwrap();
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

    let files: Vec<_> = group.get_files().collect_all().await.unwrap();
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

    let folders: Vec<_> = group.get_folders().collect_all().await.unwrap();
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

    let pages: Vec<_> = group.get_pages().collect_all().await.unwrap();
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

    let topics: Vec<_> = group.get_discussion_topics().collect_all().await.unwrap();
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

    let memberships: Vec<_> = group.get_memberships().collect_all().await.unwrap();
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

    let memberships: Vec<_> = group.get_memberships().collect_all().await.unwrap();
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
    let categories: Vec<_> = course.get_group_categories().collect_all().await.unwrap();
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
            ResponseTemplate::new(200).set_body_json(serde_json::json!([group_category_json(10)])),
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
    let categories: Vec<_> = course.get_group_categories().collect_all().await.unwrap();
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
            ResponseTemplate::new(200).set_body_json(serde_json::json!([group_category_json(10)])),
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
    let categories: Vec<_> = course.get_group_categories().collect_all().await.unwrap();
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
            ResponseTemplate::new(200).set_body_json(serde_json::json!([group_category_json(10)])),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/group_categories/10/groups"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([group_json(1), group_json(2)])),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let categories: Vec<_> = course.get_group_categories().collect_all().await.unwrap();
    let groups: Vec<_> = categories[0].get_groups().collect_all().await.unwrap();
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
        .create_page(&[("wiki_page[title]".to_string(), "New Page".to_string())])
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
        .create_discussion_topic(&[("title".to_string(), "New Discussion".to_string())])
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

    let tabs: Vec<_> = group.get_tabs().collect_all().await.unwrap();
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

    let migrations: Vec<_> = group.get_content_migrations().collect_all().await.unwrap();
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

    let exports: Vec<_> = group.get_content_exports().collect_all().await.unwrap();
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

    let folders: Vec<_> = group.resolve_path(None).collect_all().await.unwrap();
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
    let categories: Vec<_> = course.get_group_categories().collect_all().await.unwrap();
    let users: Vec<_> = categories[0].get_users().collect_all().await.unwrap();
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
    let categories: Vec<_> = course.get_group_categories().collect_all().await.unwrap();
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
        .and(path(
            "/api/v1/group_categories/10/assign_unassigned_members",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "workflow_state": "queued",
            "completion": 0
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let categories: Vec<_> = course.get_group_categories().collect_all().await.unwrap();
    let progress = categories[0].assign_members().await.unwrap();
    assert_eq!(progress.id, 5);
    assert_eq!(progress.workflow_state.as_deref(), Some("queued"));
}

#[tokio::test]
async fn test_group_show_front_page() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/front_page"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "url": "front-page",
            "title": "Front Page"
        })))
        .mount(&server)
        .await;

    let page = group.show_front_page().await.unwrap();
    assert_eq!(page.url.as_deref(), Some("front-page"));
    assert_eq!(page.title.as_deref(), Some("Front Page"));
    assert_eq!(page.group_id, Some(1));
}

#[tokio::test]
async fn test_group_edit_front_page() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/groups/1/front_page"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "url": "front-page-1",
            "title": "Front Pagest"
        })))
        .mount(&server)
        .await;

    let page = group.edit_front_page(&[]).await.unwrap();
    assert_eq!(page.url.as_deref(), Some("front-page-1"));
    assert_eq!(page.title.as_deref(), Some("Front Pagest"));
    assert_eq!(page.group_id, Some(1));
}

#[tokio::test]
async fn test_group_get_file_quota() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/files/quota"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "quota": 777648912,
            "quota_used": 567864213
        })))
        .mount(&server)
        .await;

    let quota = group.get_file_quota().await.unwrap();
    assert_eq!(quota["quota"], 777648912);
    assert_eq!(quota["quota_used"], 567864213);
}

#[tokio::test]
async fn test_group_get_external_feeds() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/external_feeds"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "display_name": "My Blog", "url": "https://example.com/myblog.rss"},
            {"id": 2, "display_name": "My Blog 2", "url": "https://example.com/myblog2.rss"}
        ])))
        .mount(&server)
        .await;

    let feeds = group.get_external_feeds().collect_all().await.unwrap();
    assert_eq!(feeds.len(), 2);
    assert_eq!(feeds[0]["display_name"], "My Blog");
}

#[tokio::test]
async fn test_group_create_external_feed() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/groups/1/external_feeds"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "display_name": "My Blog",
            "url": "https://example.com/myblog.rss"
        })))
        .mount(&server)
        .await;

    let feed = group
        .create_external_feed("https://example.com/myblog.rss")
        .await
        .unwrap();
    assert_eq!(feed["id"], 1);
    assert_eq!(feed["url"], "https://example.com/myblog.rss");
}

#[tokio::test]
async fn test_group_delete_external_feed() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/groups/1/external_feeds/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "display_name": "My Blog",
            "url": "https://example.com/myblog.rss"
        })))
        .mount(&server)
        .await;

    let feed = group.delete_external_feed(1).await.unwrap();
    assert_eq!(feed["display_name"], "My Blog");
}

#[tokio::test]
async fn test_group_get_assignment_override() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/assignments/1/override"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 30,
            "assignment_id": 1,
            "group_id": 1,
            "title": "Group Assignment Override"
        })))
        .mount(&server)
        .await;

    let override_val = group.get_assignment_override(1).await.unwrap();
    assert_eq!(override_val["id"], 30);
    assert_eq!(override_val["group_id"], 1);
    assert_eq!(override_val["title"], "Group Assignment Override");
}

#[tokio::test]
async fn test_group_set_usage_rights() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/groups/1/usage_rights"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "use_justification": "fair_use",
            "license": "private",
            "message": "2 files updated",
            "file_ids": [1, 2]
        })))
        .mount(&server)
        .await;

    let result = group.set_usage_rights(&[]).await.unwrap();
    assert_eq!(result["use_justification"], "fair_use");
    assert_eq!(result["message"], "2 files updated");
}

#[tokio::test]
async fn test_group_remove_usage_rights() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/groups/1/usage_rights"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "2 files updated",
            "file_ids": [1, 2]
        })))
        .mount(&server)
        .await;

    let result = group.remove_usage_rights(&[]).await.unwrap();
    assert_eq!(result["message"], "2 files updated");
    assert_eq!(result["file_ids"][0], 1);
}

#[tokio::test]
async fn test_group_get_licenses() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/content_licenses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "private", "name": "Private (Copyrighted)", "url": "http://en.wikipedia.org/wiki/Copyright"},
            {"id": "public_domain", "name": "Public domain", "url": "http://en.wikipedia.org/wiki/Public_domain"}
        ])))
        .mount(&server)
        .await;

    let licenses = group.get_licenses().collect_all().await.unwrap();
    assert_eq!(licenses.len(), 2);
    assert_eq!(licenses[0]["id"], "private");
    assert_eq!(licenses[1]["id"], "public_domain");
}

#[tokio::test]
async fn test_group_resolve_path_with_value() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/groups/1/folders/by_path/Folder_Level_1/Folder_Level_2/Folder_Level_3",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 2, "name": "files", "full_name": "files"},
            {"id": 3, "name": "Folder_Level_1", "full_name": "files/Folder_Level_1"},
            {"id": 4, "name": "Folder_Level_2", "full_name": "files/Folder_Level_1/Folder_Level_2"},
            {"id": 5, "name": "Folder_Level_3", "full_name": "files/Folder_Level_1/Folder_Level_2/Folder_Level_3"}
        ])))
        .mount(&server)
        .await;

    let folders = group
        .resolve_path(Some("Folder_Level_1/Folder_Level_2/Folder_Level_3"))
        .collect_all()
        .await
        .unwrap();
    assert_eq!(folders.len(), 4);
    assert_eq!(folders[0].name.as_deref(), Some("files"));
    assert_eq!(folders[3].name.as_deref(), Some("Folder_Level_3"));
}

// ---- Batch 4: remaining depth ----

#[tokio::test]
async fn test_group_get_content_migration() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/content_migrations/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "migration_type": "dummy_importer",
            "workflow_state": "completed"
        })))
        .mount(&server)
        .await;

    let m = group.get_content_migration(1).await.unwrap();
    assert_eq!(m["id"], 1);
    assert_eq!(m["workflow_state"], "completed");
}

#[tokio::test]
async fn test_group_create_content_migration() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/groups/1/content_migrations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "migration_type": "dummy_importer",
            "workflow_state": "created"
        })))
        .mount(&server)
        .await;

    let m = group
        .create_content_migration("dummy_importer")
        .await
        .unwrap();
    assert_eq!(m["migration_type"], "dummy_importer");
    assert_eq!(m["workflow_state"], "created");
}

#[tokio::test]
async fn test_group_get_migration_systems() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/content_migrations/migrators"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"type": "dummy_importer", "requires_file_upload": false, "name": "Dummy"}
        ])))
        .mount(&server)
        .await;

    let systems = group.get_migration_systems().collect_all().await.unwrap();
    assert_eq!(systems.len(), 1);
    assert_eq!(systems[0]["type"], "dummy_importer");
}

#[tokio::test]
async fn test_group_get_content_export() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/content_exports/11"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 11,
            "export_type": "common_cartridge",
            "workflow_state": "exported"
        })))
        .mount(&server)
        .await;

    let e = group.get_content_export(11).await.unwrap();
    assert_eq!(e["id"], 11);
    assert_eq!(e["export_type"], "common_cartridge");
}

#[tokio::test]
async fn test_group_export_content() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/groups/1/content_exports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99,
            "export_type": "common_cartridge",
            "workflow_state": "created"
        })))
        .mount(&server)
        .await;

    let e = group.export_content("common_cartridge").await.unwrap();
    assert_eq!(e["export_type"], "common_cartridge");
}

#[tokio::test]
async fn test_group_get_full_discussion_topic() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/discussion_topics/5/view"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "view": [{"id": 1}],
            "participants": [{"id": 10}]
        })))
        .mount(&server)
        .await;

    let t = group.get_full_discussion_topic(5).await.unwrap();
    assert_eq!(t["id"], 5);
    assert!(t.get("view").is_some());
}

#[tokio::test]
async fn test_group_get_activity_stream_summary() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/1/activity_stream/summary"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"type": "DiscussionTopic", "unread_count": 2, "count": 7}
        ])))
        .mount(&server)
        .await;

    let summary = group.get_activity_stream_summary().await.unwrap();
    assert!(summary.is_array());
}

#[tokio::test]
async fn test_group_reorder_pinned_topics() {
    let server = MockServer::start().await;
    let group = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/groups/1/discussion_topics/reorder"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "reorder": true,
            "order": [1, 2, 3]
        })))
        .mount(&server)
        .await;

    let result = group.reorder_pinned_topics(&[1, 2, 3]).await.unwrap();
    assert_eq!(result["reorder"], true);
}
