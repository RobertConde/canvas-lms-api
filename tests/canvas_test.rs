use canvas_lms_api::resources::params::course_params::CreateCourseParams;
use canvas_lms_api::resources::params::user_params::CreateUserParams;
use canvas_lms_api::resources::user::UserId;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_create_course() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/courses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99,
            "name": "New Course",
            "course_code": "NEW-101"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let params = CreateCourseParams {
        name: Some("New Course".to_string()),
        course_code: Some("NEW-101".to_string()),
        ..Default::default()
    };
    let course = canvas.create_course(1, params).await.unwrap();

    assert_eq!(course.id, 99);
    assert_eq!(course.name.as_deref(), Some("New Course"));
    assert_eq!(course.course_code.as_deref(), Some("NEW-101"));
}

#[tokio::test]
async fn test_delete_course() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "Deleted Course",
            "workflow_state": "deleted"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.delete_course(1).await.unwrap();

    assert_eq!(course.id, 1);
    assert!(matches!(
        course.workflow_state,
        Some(canvas_lms_api::resources::types::WorkflowState::Deleted)
    ));
}

#[tokio::test]
async fn test_get_user() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "name": "Alice"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let user = canvas.get_user(UserId::Id(42)).await.unwrap();

    assert_eq!(user.id, 42);
    assert_eq!(user.name.as_deref(), Some("Alice"));
}

#[tokio::test]
async fn test_get_current_user() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/self"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "name": "Bob",
            "effective_locale": "en"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let user = canvas.get_current_user().await.unwrap();

    assert_eq!(user.id, 7);
    assert_eq!(user.name.as_deref(), Some("Bob"));
    assert_eq!(user.effective_locale.as_deref(), Some("en"));
}

#[tokio::test]
async fn test_create_user() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 55,
            "name": "Charlie"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let params = CreateUserParams {
        name: "Charlie".to_string(),
        ..Default::default()
    };
    let user = canvas.create_user(1, params).await.unwrap();

    assert_eq!(user.id, 55);
    assert_eq!(user.name.as_deref(), Some("Charlie"));
}

#[tokio::test]
async fn test_get_account() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "Root Account"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let account = canvas.get_account(1).await.unwrap();

    assert_eq!(account.id, 1);
    assert_eq!(account.name.as_deref(), Some("Root Account"));
}

#[tokio::test]
async fn test_get_accounts() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "Root Account"},
            {"id": 2, "name": "Sub Account"}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let accounts = canvas.get_accounts().collect_all().await.unwrap();

    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[0].id, 1);
    assert_eq!(accounts[1].id, 2);
}

#[tokio::test]
async fn test_get_section() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/sections/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10, "name": "Section A", "course_id": 1
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let s = canvas.get_section(10).await.unwrap();
    assert_eq!(s.id, 10);
    assert_eq!(s.name.as_deref(), Some("Section A"));
}

#[tokio::test]
async fn test_get_group() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/groups/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20, "name": "Study Group"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let g = canvas.get_group(20).await.unwrap();
    assert_eq!(g.id, 20);
    assert_eq!(g.name.as_deref(), Some("Study Group"));
}

#[tokio::test]
async fn test_get_file() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/files/30"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 30, "display_name": "notes.pdf", "size": 1024
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let f = canvas.get_file(30).await.unwrap();
    assert_eq!(f.id, 30);
    assert_eq!(f.display_name.as_deref(), Some("notes.pdf"));
}

#[tokio::test]
async fn test_get_folder() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/folders/40"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 40, "name": "Homework", "full_name": "course files/Homework"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let f = canvas.get_folder(40).await.unwrap();
    assert_eq!(f.id, 40);
    assert_eq!(f.name.as_deref(), Some("Homework"));
}

#[tokio::test]
async fn test_get_progress() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/progress/50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 50, "workflow_state": "running", "completion": 42
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let p = canvas.get_progress(50).await.unwrap();
    assert_eq!(p.id, 50);
    assert_eq!(p.workflow_state.as_deref(), Some("running"));
}

#[tokio::test]
async fn test_get_outcome() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/outcomes/15"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 15,
            "title": "Written Communication",
            "points_possible": 5.0
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let outcome = canvas.get_outcome(15).await.unwrap();
    assert_eq!(outcome.id, 15);
    assert_eq!(outcome.title.as_deref(), Some("Written Communication"));
    assert_eq!(outcome.points_possible, Some(5.0));
}


#[tokio::test]
async fn test_get_group_category() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/group_categories/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "name": "Project Groups",
            "role": "student_organized"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let gc = canvas.get_group_category(5).await.unwrap();
    assert_eq!(gc.id, 5);
    assert_eq!(gc.name.as_deref(), Some("Project Groups"));
}

#[tokio::test]
async fn test_get_account_calendars() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/account_calendars"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "Root Calendar", "visible": true},
            {"id": 2, "name": "Sub Calendar", "visible": false}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let cals = canvas.get_account_calendars().collect_all().await.unwrap();
    assert_eq!(cals.len(), 2);
    assert_eq!(cals[0].id, Some(1));
    assert_eq!(cals[0].name.as_deref(), Some("Root Calendar"));
    assert_eq!(cals[1].visible, Some(false));
}

#[tokio::test]
async fn test_get_root_outcome_group() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/global/root_outcome_group"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "title": "Root Outcomes",
            "context_type": null
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let og = canvas.get_root_outcome_group().await.unwrap();
    assert_eq!(og.id, 1);
    assert_eq!(og.title.as_deref(), Some("Root Outcomes"));
}

#[tokio::test]
async fn test_get_announcements() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/announcements"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "title": "Welcome", "discussion_type": "side_comment"},
            {"id": 11, "title": "Exam reminder"}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let items = canvas
        .get_announcements(&["course_1", "course_2"])
        .collect_all()
        .await
        .unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["id"], 10);
    assert_eq!(items[1]["title"], "Exam reminder");
}

#[tokio::test]
async fn test_search_accounts() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"name": "University of Test", "domain": "test.instructure.com"}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let result = canvas.search_accounts().await.unwrap();
    assert!(result.is_array());
    assert_eq!(result[0]["name"], "University of Test");
}

#[tokio::test]
async fn test_search_all_courses() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/search/all_courses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "Intro to Rust"}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let result = canvas.search_all_courses().await.unwrap();
    assert!(result.is_array());
    assert_eq!(result[0]["name"], "Intro to Rust");
}

#[tokio::test]
async fn test_search_recipients() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/search/recipients"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "1", "name": "Alice"}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let result = canvas.search_recipients().await.unwrap();
    assert!(result.is_array());
    assert_eq!(result[0]["name"], "Alice");
}

#[tokio::test]
async fn test_get_activity_stream_summary() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/users/self/activity_stream/summary"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"type": "DiscussionTopic", "count": 3, "unread_count": 1}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let result = canvas.get_activity_stream_summary().await.unwrap();
    assert!(result.is_array());
    assert_eq!(result[0]["type"], "DiscussionTopic");
}

#[tokio::test]
async fn test_get_todo_items() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/users/self/todo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"type": "submitting", "assignment": {"id": 1, "name": "Homework 1"}}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let items = canvas.get_todo_items().collect_all().await.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["type"], "submitting");
}

#[tokio::test]
async fn test_get_upcoming_events() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/users/self/upcoming_events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 100, "title": "Assignment due", "type": "assignment"}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let result = canvas.get_upcoming_events().await.unwrap();
    assert!(result.is_array());
    assert_eq!(result[0]["id"], 100);
}

#[tokio::test]
async fn test_get_course_accounts() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/course_accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "Teaching Account", "workflow_state": "active"}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let accounts = canvas.get_course_accounts().collect_all().await.unwrap();
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].id, 1);
    assert_eq!(accounts[0].name.as_deref(), Some("Teaching Account"));
}

#[tokio::test]
async fn test_get_course_nickname() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/users/self/course_nicknames/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "course_id": 99,
            "name": "Intro to Biology",
            "nickname": "Bio 101"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let result = canvas.get_course_nickname(99).await.unwrap();
    assert_eq!(result["course_id"], 99);
    assert_eq!(result["nickname"], "Bio 101");
}

#[tokio::test]
async fn test_get_course_nicknames() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/users/self/course_nicknames"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"course_id": 1, "nickname": "Math"},
            {"course_id": 2, "nickname": "Sci"}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let nicknames = canvas.get_course_nicknames().collect_all().await.unwrap();
    assert_eq!(nicknames.len(), 2);
    assert_eq!(nicknames[0]["nickname"], "Math");
    assert_eq!(nicknames[1]["nickname"], "Sci");
}

#[tokio::test]
async fn test_set_course_nickname() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/users/self/course_nicknames/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "course_id": 5,
            "name": "Calculus II",
            "nickname": "Calc"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let result = canvas.set_course_nickname(5, "Calc").await.unwrap();
    assert_eq!(result["course_id"], 5);
    assert_eq!(result["nickname"], "Calc");
}

#[tokio::test]
async fn test_clear_course_nicknames() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/users/self/course_nicknames"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "OK"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let result = canvas.clear_course_nicknames().await.unwrap();
    assert_eq!(result["message"], "OK");
}

#[tokio::test]
async fn test_get_epub_exports() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/epub_exports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"course_id": 1, "epub_export": {"id": 10, "workflow_state": "generated"}}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let exports = canvas.get_epub_exports().collect_all().await.unwrap();
    assert_eq!(exports.len(), 1);
    assert_eq!(exports[0]["course_id"], 1);
}

#[tokio::test]
async fn test_get_brand_variables() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/brand_variables"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ic-brand-primary": "#E66000",
            "ic-link-color": "#0770A3"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let result = canvas.get_brand_variables().await.unwrap();
    assert_eq!(result["ic-brand-primary"], "#E66000");
}

#[tokio::test]
async fn test_get_comm_messages() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/comm_messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "subject": "Assignment due", "author": "Canvas"}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let messages = canvas.get_comm_messages(42).collect_all().await.unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["subject"], "Assignment due");
}

#[tokio::test]
async fn test_conversations_batch_update() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/conversations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 55,
            "workflow_state": "queued",
            "completion": 0
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let progress = canvas
        .conversations_batch_update(&[1, 2, 3], "mark_as_read")
        .await
        .unwrap();
    assert_eq!(progress.id, 55);
}

#[tokio::test]
async fn test_conversations_get_running_batches() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/conversations/batches"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let result = canvas.conversations_get_running_batches().await.unwrap();
    assert!(result.is_array());
}

#[tokio::test]
async fn test_conversations_mark_all_as_read() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/conversations/mark_all_as_read"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas.conversations_mark_all_as_read().await.unwrap();
}

#[tokio::test]
async fn test_conversations_unread_count() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/conversations/unread_count"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "unread_count": "7"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let result = canvas.conversations_unread_count().await.unwrap();
    assert_eq!(result["unread_count"], "7");
}

#[tokio::test]
async fn test_get_group_participants() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/appointment_groups/10/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "Study Group A"},
            {"id": 2, "name": "Study Group B"}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let groups = canvas.get_group_participants(10).collect_all().await.unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].id, 1);
    assert_eq!(groups[0].name.as_deref(), Some("Study Group A"));
}

#[tokio::test]
async fn test_get_user_participants() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/appointment_groups/10/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 101, "name": "Alice"},
            {"id": 102, "name": "Bob"}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let users = canvas.get_user_participants(10).collect_all().await.unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].id, 101);
    assert_eq!(users[0].name.as_deref(), Some("Alice"));
}

#[tokio::test]
async fn test_reserve_time_slot_no_participant() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/calendar_events/77/reservations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 200,
            "title": "Office Hours",
            "context_code": "course_1"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let event = canvas.reserve_time_slot(77, None).await.unwrap();
    assert_eq!(event.id, 200);
    assert_eq!(event.title.as_deref(), Some("Office Hours"));
}

#[tokio::test]
async fn test_reserve_time_slot_with_participant() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/calendar_events/77/reservations/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 201,
            "title": "Office Hours",
            "context_code": "course_1"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let event = canvas.reserve_time_slot(77, Some(99)).await.unwrap();
    assert_eq!(event.id, 201);
}
