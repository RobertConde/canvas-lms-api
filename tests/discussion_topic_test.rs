use canvas_lms_api::{
    resources::discussion_topic::{PostEntryParams, UpdateDiscussionParams},
    Canvas,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn topic_json(id: u64, course_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "course_id": course_id,
        "title": "Test Discussion",
        "message": "Hello world",
        "published": true
    })
}

fn entry_json(id: u64, user_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "user_id": user_id,
        "message": "Top Level Entry",
        "created_at": "2024-01-01T00:00:00Z"
    })
}

async fn setup(
    server: &MockServer,
) -> canvas_lms_api::resources::discussion_topic::DiscussionTopic {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/discussion_topics/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(topic_json(1, 1)))
        .mount(server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    course.get_discussion_topic(1).await.unwrap()
}

async fn setup_with_entry(
    server: &MockServer,
) -> canvas_lms_api::resources::discussion_topic::DiscussionEntry {
    let topic = setup(server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/discussion_topics/1/entry_list"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([entry_json(1, 42)])),
        )
        .mount(server)
        .await;

    topic.get_entries(&[1]).await.unwrap().remove(0)
}

#[tokio::test]
async fn test_discussion_topic_update() {
    let server = MockServer::start().await;
    let topic = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/discussion_topics/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "course_id": 1,
            "title": "Updated Title"
        })))
        .mount(&server)
        .await;

    let updated = topic
        .update(UpdateDiscussionParams {
            title: Some("Updated Title".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.title.as_deref(), Some("Updated Title"));
}

#[tokio::test]
async fn test_discussion_topic_delete() {
    let server = MockServer::start().await;
    let topic = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/discussion_topics/1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    topic.delete().await.unwrap();
}

#[tokio::test]
async fn test_discussion_topic_post_entry() {
    let server = MockServer::start().await;
    let topic = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/discussion_topics/1/entries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(entry_json(10, 42)))
        .mount(&server)
        .await;

    let entry = topic
        .post_entry(PostEntryParams {
            message: Some("Hello!".to_string()),
        })
        .await
        .unwrap();
    assert_eq!(entry.id, 10);
    assert_eq!(entry.topic_id, Some(1));
    assert_eq!(entry.course_id, Some(1));
}

#[tokio::test]
async fn test_discussion_topic_get_topic_entries() {
    let server = MockServer::start().await;
    let topic = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/discussion_topics/1/entries"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([entry_json(1, 42), entry_json(2, 43)])),
        )
        .mount(&server)
        .await;

    let entries: Vec<_> = topic.get_topic_entries().collect_all().await.unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].id, 1);
    assert_eq!(entries[0].user_id, Some(42));
    assert_eq!(entries[0].course_id, Some(1));
}

#[tokio::test]
async fn test_discussion_topic_get_entries() {
    let server = MockServer::start().await;
    let topic = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/discussion_topics/1/entry_list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            entry_json(1, 42),
            entry_json(2, 43),
            {"id": 3, "user_id": 44, "message": "Lower level entry"}
        ])))
        .mount(&server)
        .await;

    let entries = topic.get_entries(&[1, 2, 3]).await.unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[2].id, 3);
    assert_eq!(entries[2].message.as_deref(), Some("Lower level entry"));
}

#[tokio::test]
async fn test_discussion_topic_mark_as_read() {
    let server = MockServer::start().await;
    let topic = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/discussion_topics/1/read"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    topic.mark_as_read().await.unwrap();
}

#[tokio::test]
async fn test_discussion_topic_mark_as_unread() {
    let server = MockServer::start().await;
    let topic = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/discussion_topics/1/read"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    topic.mark_as_unread().await.unwrap();
}

#[tokio::test]
async fn test_discussion_topic_mark_entries_as_read() {
    let server = MockServer::start().await;
    let topic = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/discussion_topics/1/read_all"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    topic.mark_entries_as_read(false).await.unwrap();
}

#[tokio::test]
async fn test_discussion_topic_mark_entries_as_unread() {
    let server = MockServer::start().await;
    let topic = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/discussion_topics/1/read_all"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    topic.mark_entries_as_unread(false).await.unwrap();
}

#[tokio::test]
async fn test_discussion_topic_subscribe() {
    let server = MockServer::start().await;
    let topic = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/discussion_topics/1/subscribed"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    topic.subscribe().await.unwrap();
}

#[tokio::test]
async fn test_discussion_topic_unsubscribe() {
    let server = MockServer::start().await;
    let topic = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/discussion_topics/1/subscribed"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    topic.unsubscribe().await.unwrap();
}

#[tokio::test]
async fn test_course_create_discussion_topic() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/discussion_topics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(topic_json(5, 1)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let topic = course
        .create_discussion_topic(UpdateDiscussionParams {
            title: Some("Test Discussion".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(topic.id, 5);
    assert_eq!(topic.course_id_ctx, Some(1));
}

// DiscussionEntry tests

#[tokio::test]
async fn test_entry_update() {
    let server = MockServer::start().await;
    let entry = setup_with_entry(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/discussion_topics/1/entries/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "user_id": 42,
            "message": "Updated message"
        })))
        .mount(&server)
        .await;

    let updated = entry.update("Updated message").await.unwrap();
    assert_eq!(updated.message.as_deref(), Some("Updated message"));
}

#[tokio::test]
async fn test_entry_delete() {
    let server = MockServer::start().await;
    let entry = setup_with_entry(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/discussion_topics/1/entries/1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    entry.delete().await.unwrap();
}

#[tokio::test]
async fn test_entry_post_reply() {
    let server = MockServer::start().await;
    let entry = setup_with_entry(&server).await;

    Mock::given(method("POST"))
        .and(path(
            "/api/v1/courses/1/discussion_topics/1/entries/1/replies",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "user_id": 99,
            "message": "Reply message 1"
        })))
        .mount(&server)
        .await;

    let reply = entry.post_reply("Reply message 1").await.unwrap();
    assert_eq!(reply.id, 5);
    assert_eq!(reply.message.as_deref(), Some("Reply message 1"));
}

#[tokio::test]
async fn test_entry_get_replies() {
    let server = MockServer::start().await;
    let entry = setup_with_entry(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/courses/1/discussion_topics/1/entries/1/replies",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 5, "user_id": 99, "message": "Reply message 1"},
            {"id": 6, "user_id": 100, "message": "Reply message 2"}
        ])))
        .mount(&server)
        .await;

    let replies: Vec<_> = entry.get_replies().collect_all().await.unwrap();
    assert_eq!(replies.len(), 2);
    assert_eq!(replies[0].id, 5);
    assert_eq!(replies[0].message.as_deref(), Some("Reply message 1"));
}

#[tokio::test]
async fn test_entry_mark_as_read() {
    let server = MockServer::start().await;
    let entry = setup_with_entry(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/discussion_topics/1/entries/1/read"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    entry.mark_as_read().await.unwrap();
}

#[tokio::test]
async fn test_entry_mark_as_unread() {
    let server = MockServer::start().await;
    let entry = setup_with_entry(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/discussion_topics/1/entries/1/read"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    entry.mark_as_unread().await.unwrap();
}

#[tokio::test]
async fn test_entry_rate_valid() {
    let server = MockServer::start().await;
    let entry = setup_with_entry(&server).await;

    Mock::given(method("POST"))
        .and(path(
            "/api/v1/courses/1/discussion_topics/1/entries/1/rating",
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    entry.rate(1).await.unwrap();
}

#[tokio::test]
async fn test_entry_rate_invalid() {
    let server = MockServer::start().await;
    let entry = setup_with_entry(&server).await;

    let result = entry.rate(2).await;
    assert!(result.is_err());
}


#[tokio::test]
async fn test_discussion_topic_get_parent_course() {
    let server = MockServer::start().await;
    let topic = setup(&server).await;

    // GET /courses/1 is already registered by setup(); get_parent() re-uses it.
    let parent = topic.get_parent().await.unwrap();
    assert_eq!(parent["id"], 1);
}
