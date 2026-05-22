use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup(server: &MockServer) -> canvas_lms_api::resources::course::Course {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"id": 1, "name": "Test Course"})),
        )
        .mount(server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas.get_course(1).await.unwrap()
}

#[tokio::test]
async fn test_course_conclude() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "delete": true
        })))
        .mount(&server)
        .await;

    let result = course.conclude().await.unwrap();
    assert!(result.is_object());
}

#[tokio::test]
async fn test_course_reset() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/reset_content"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 2,
            "name": "Copy of Test Course"
        })))
        .mount(&server)
        .await;

    let new_course = course.reset().await.unwrap();
    assert_eq!(new_course.id, 2);
}

#[tokio::test]
async fn test_course_get_settings() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/settings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "allow_student_discussion_topics": true,
            "allow_student_forum_attachments": false
        })))
        .mount(&server)
        .await;

    let settings = course.get_settings().await.unwrap();
    assert_eq!(settings["allow_student_discussion_topics"], true);
}

#[tokio::test]
async fn test_course_update_settings() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/settings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "allow_student_discussion_topics": false
        })))
        .mount(&server)
        .await;

    let updated = course
        .update_settings(&[(
            "allow_student_discussion_topics".to_string(),
            "false".to_string(),
        )])
        .await
        .unwrap();
    assert_eq!(updated["allow_student_discussion_topics"], false);
}

#[tokio::test]
async fn test_course_get_late_policy() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/late_policy"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "late_policy": {
                "id": 1,
                "course_id": 1,
                "late_submission_deduction_enabled": true,
                "late_submission_deduction": 10.0
            }
        })))
        .mount(&server)
        .await;

    let policy = course.get_late_policy().await.unwrap();
    assert!(policy.get("late_policy").is_some());
}

#[tokio::test]
async fn test_course_get_multiple_submissions() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/students/submissions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "user_id": 3, "assignment_id": 2, "grade": "A"},
            {"id": 11, "user_id": 4, "assignment_id": 2, "grade": "B"}
        ])))
        .mount(&server)
        .await;

    let subs: Vec<_> = course
        .get_multiple_submissions()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(subs.len(), 2);
    assert_eq!(subs[0].id, 10);
    assert_eq!(subs[0].course_id, Some(1));
}

#[tokio::test]
async fn test_course_enroll_user() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/enrollments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 50,
            "course_id": 1,
            "user_id": 42,
            "type": "StudentEnrollment",
            "enrollment_state": "active"
        })))
        .mount(&server)
        .await;

    let enrollment = course.enroll_user(42, "StudentEnrollment").await.unwrap();
    assert_eq!(enrollment.id, 50);
    assert_eq!(enrollment.course_id, Some(1));
}

#[tokio::test]
async fn test_course_submissions_bulk_update() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/submissions/update_grades"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99,
            "workflow_state": "queued",
            "completion": 0
        })))
        .mount(&server)
        .await;

    let progress = course
        .submissions_bulk_update(&[(
            "grade_data[3][2][posted_grade]".to_string(),
            "A".to_string(),
        )])
        .await
        .unwrap();
    assert_eq!(progress.id, 99);
    assert_eq!(progress.workflow_state.as_deref(), Some("queued"));
}

// ---- Front page ----

#[tokio::test]
async fn test_course_show_front_page() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/front_page"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "url": "front-page",
            "title": "Front Page",
            "body": "<p>Welcome</p>"
        })))
        .mount(&server)
        .await;

    let page = course.show_front_page().await.unwrap();
    assert_eq!(page.title.as_deref(), Some("Front Page"));
    assert_eq!(page.course_id, Some(1));
}

#[tokio::test]
async fn test_course_edit_front_page() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/front_page"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "url": "front-page",
            "title": "Updated Front Page",
            "body": "<p>Updated</p>"
        })))
        .mount(&server)
        .await;

    let page = course
        .edit_front_page(&[(
            "wiki_page[title]".to_string(),
            "Updated Front Page".to_string(),
        )])
        .await
        .unwrap();
    assert_eq!(page.title.as_deref(), Some("Updated Front Page"));
    assert_eq!(page.course_id, Some(1));
}

// ---- Content ----

#[tokio::test]
async fn test_course_export_content() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/content_exports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 111,
            "export_type": "common_cartridge",
            "workflow_state": "created"
        })))
        .mount(&server)
        .await;

    let export = course.export_content("common_cartridge").await.unwrap();
    assert_eq!(export.id, 111);
    assert_eq!(export.export_type.as_deref(), Some("common_cartridge"));
}

#[tokio::test]
async fn test_course_get_full_discussion_topic() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/discussion_topics/5/view"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "view": [{"id": 1, "message": "Hello"}],
            "participants": [{"id": 10, "display_name": "Alice"}]
        })))
        .mount(&server)
        .await;

    let topic = course.get_full_discussion_topic(5).await.unwrap();
    assert_eq!(topic["id"], 5);
    assert!(topic.get("view").is_some());
    assert!(topic.get("participants").is_some());
}

#[tokio::test]
async fn test_course_preview_html() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/preview_html"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "html": "<p>hello</p>"
        })))
        .mount(&server)
        .await;

    let html = course
        .preview_html("<script></script><p>hello</p>")
        .await
        .unwrap();
    assert_eq!(html, "<p>hello</p>");
}

#[tokio::test]
async fn test_course_reorder_pinned_topics() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/discussion_topics/reorder"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "reorder": true,
            "order": [1, 2, 3]
        })))
        .mount(&server)
        .await;

    let result = course.reorder_pinned_topics(&[1, 2, 3]).await.unwrap();
    assert_eq!(result["reorder"], true);
}

// ---- Users ----

#[tokio::test]
async fn test_course_get_user() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/users/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "name": "Alice"
        })))
        .mount(&server)
        .await;

    let user = course.get_user(42).await.unwrap();
    assert_eq!(user.id, 42);
    assert_eq!(user.name.as_deref(), Some("Alice"));
}

#[tokio::test]
async fn test_course_get_recent_students() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/recent_students"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "John Doe"},
            {"id": 2, "name": "Jane Doe"}
        ])))
        .mount(&server)
        .await;

    let students = course.get_recent_students().collect_all().await.unwrap();
    assert_eq!(students.len(), 2);
    assert_eq!(students[0].id, 1);
}

// ---- Usage rights / licenses ----

#[tokio::test]
async fn test_course_set_usage_rights() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/usage_rights"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "use_justification": "fair_use",
            "license": "private",
            "message": "2 files updated",
            "file_ids": [1, 2]
        })))
        .mount(&server)
        .await;

    let result = course
        .set_usage_rights(&[
            ("file_ids[]".to_string(), "1".to_string()),
            ("file_ids[]".to_string(), "2".to_string()),
            (
                "usage_rights[use_justification]".to_string(),
                "fair_use".to_string(),
            ),
        ])
        .await
        .unwrap();
    assert_eq!(result["use_justification"], "fair_use");
    assert_eq!(result["message"], "2 files updated");
}

#[tokio::test]
async fn test_course_remove_usage_rights() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/usage_rights"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "2 files updated",
            "file_ids": [1, 2]
        })))
        .mount(&server)
        .await;

    let result = course
        .remove_usage_rights(&[
            ("file_ids[]".to_string(), "1".to_string()),
            ("file_ids[]".to_string(), "2".to_string()),
        ])
        .await
        .unwrap();
    assert_eq!(result["message"], "2 files updated");
}

#[tokio::test]
async fn test_course_get_licenses() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/content_licenses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "private", "name": "Private (Copyrighted)", "url": "https://copyright.gov"},
            {"id": "public_domain", "name": "Public Domain", "url": "https://creativecommons.org/licenses/publicdomain/"}
        ])))
        .mount(&server)
        .await;

    let licenses = course.get_licenses().collect_all().await.unwrap();
    assert_eq!(licenses.len(), 2);
    assert_eq!(licenses[0]["id"], "private");
}

// ---- External feeds ----

#[tokio::test]
async fn test_course_get_external_feeds() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/external_feeds"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "display_name": "My Blog", "url": "https://example.com/blog.rss"},
            {"id": 2, "display_name": "News", "url": "https://example.com/news.rss"}
        ])))
        .mount(&server)
        .await;

    let feeds = course.get_external_feeds().collect_all().await.unwrap();
    assert_eq!(feeds.len(), 2);
    assert_eq!(feeds[0]["display_name"], "My Blog");
}

#[tokio::test]
async fn test_course_create_external_feed() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/external_feeds"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
            "display_name": "New Feed",
            "url": "https://example.com/myblog.rss"
        })))
        .mount(&server)
        .await;

    let feed = course
        .create_external_feed("https://example.com/myblog.rss")
        .await
        .unwrap();
    assert_eq!(feed["id"], 3);
    assert_eq!(feed["url"], "https://example.com/myblog.rss");
}

#[tokio::test]
async fn test_course_delete_external_feed() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/external_feeds/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "display_name": "My Blog",
            "url": "https://example.com/blog.rss"
        })))
        .mount(&server)
        .await;

    let deleted = course.delete_external_feed(1).await.unwrap();
    assert_eq!(deleted["id"], 1);
    assert_eq!(deleted["display_name"], "My Blog");
}

// ---- Sections ----

#[tokio::test]
async fn test_course_create_course_section() {
    let server = MockServer::start().await;
    let course = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/sections"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "name": "New Section",
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let section = course
        .create_course_section(&[(
            "course_section[name]".to_string(),
            "New Section".to_string(),
        )])
        .await
        .unwrap();
    assert_eq!(section.id, 10);
    assert_eq!(section.name.as_deref(), Some("New Section"));
    assert_eq!(section.course_id, Some(1));
}
