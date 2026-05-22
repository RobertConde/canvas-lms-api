use canvas_lms_api::{
    resources::assignment::{
        AssignmentGroupParams, AssignmentOverrideParams, AssignmentParams, SubmitAssignmentParams,
    },
    Canvas,
};
use futures::StreamExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn assignment_json(id: u64, course_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "course_id": course_id,
        "name": "Assignment 1",
        "points_possible": 100.0,
        "submission_types": ["online_upload"]
    })
}

fn submission_json(id: u64, course_id: u64, assignment_id: u64, user_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "course_id": course_id,
        "assignment_id": assignment_id,
        "user_id": user_id,
        "workflow_state": "submitted"
    })
}

fn override_json(id: u64, assignment_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "assignment_id": assignment_id,
        "title": "Override 1",
        "course_section_id": 5
    })
}

fn group_json(id: u64, course_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "course_id": course_id,
        "name": "Group 1",
        "group_weight": 50.0,
        "position": 1
    })
}

async fn setup(server: &MockServer) -> canvas_lms_api::resources::assignment::Assignment {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignments/2"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(assignment_json(2, 1)),
        )
        .mount(server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    course.get_assignment(2).await.unwrap()
}

#[tokio::test]
async fn test_assignment_edit() {
    let server = MockServer::start().await;
    let assignment = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/assignments/2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 2,
            "course_id": 1,
            "name": "Updated Assignment"
        })))
        .mount(&server)
        .await;

    let updated = assignment
        .edit(AssignmentParams {
            name: Some("Updated Assignment".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.name.as_deref(), Some("Updated Assignment"));
    assert_eq!(updated.course_id, Some(1));
}

#[tokio::test]
async fn test_assignment_delete() {
    let server = MockServer::start().await;
    let assignment = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/assignments/2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(assignment_json(2, 1)))
        .mount(&server)
        .await;

    let deleted = assignment.delete().await.unwrap();
    assert_eq!(deleted.id, 2);
}

#[tokio::test]
async fn test_assignment_get_submissions() {
    let server = MockServer::start().await;
    let assignment = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignments/2/submissions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            submission_json(10, 1, 2, 3),
            submission_json(11, 1, 2, 4)
        ])))
        .mount(&server)
        .await;

    let subs: Vec<_> = assignment
        .get_submissions()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(subs.len(), 2);
    assert_eq!(subs[0].id, 10);
    assert_eq!(subs[0].course_id, Some(1));
}

#[tokio::test]
async fn test_assignment_get_submission() {
    let server = MockServer::start().await;
    let assignment = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignments/2/submissions/3"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(submission_json(10, 1, 2, 3)),
        )
        .mount(&server)
        .await;

    let sub = assignment.get_submission(3).await.unwrap();
    assert_eq!(sub.id, 10);
    assert_eq!(sub.course_id, Some(1));
}

#[tokio::test]
async fn test_assignment_submit() {
    let server = MockServer::start().await;
    let assignment = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/assignments/2/submissions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(submission_json(12, 1, 2, 5)),
        )
        .mount(&server)
        .await;

    let sub = assignment
        .submit(SubmitAssignmentParams {
            submission_type: "online_text_entry".to_string(),
            body: Some("My answer".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(sub.id, 12);
}

#[tokio::test]
async fn test_assignment_get_overrides() {
    let server = MockServer::start().await;
    let assignment = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignments/2/overrides"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            override_json(20, 2),
            override_json(21, 2)
        ])))
        .mount(&server)
        .await;

    let overrides: Vec<_> = assignment
        .get_overrides()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(overrides.len(), 2);
    assert_eq!(overrides[0].id, 20);
    assert_eq!(overrides[0].course_id, Some(1));
}

#[tokio::test]
async fn test_assignment_get_override() {
    let server = MockServer::start().await;
    let assignment = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignments/2/overrides/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(override_json(20, 2)))
        .mount(&server)
        .await;

    let ov = assignment.get_override(20).await.unwrap();
    assert_eq!(ov.id, 20);
    assert_eq!(ov.assignment_id, Some(2));
    assert_eq!(ov.course_id, Some(1));
}

#[tokio::test]
async fn test_assignment_create_override() {
    let server = MockServer::start().await;
    let assignment = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/assignments/2/overrides"))
        .respond_with(ResponseTemplate::new(200).set_body_json(override_json(22, 2)))
        .mount(&server)
        .await;

    let ov = assignment
        .create_override(AssignmentOverrideParams {
            course_section_id: Some(5),
            title: Some("Override 1".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(ov.id, 22);
    assert_eq!(ov.course_id, Some(1));
}

#[tokio::test]
async fn test_assignment_get_peer_reviews() {
    let server = MockServer::start().await;
    let assignment = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignments/2/peer_reviews"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"assessor_id": 10, "user_id": 3},
            {"assessor_id": 11, "user_id": 4}
        ])))
        .mount(&server)
        .await;

    let prs: Vec<_> = assignment
        .get_peer_reviews()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(prs.len(), 2);
}

#[tokio::test]
async fn test_assignment_get_gradeable_students() {
    let server = MockServer::start().await;
    let assignment = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignments/2/gradeable_students"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 3, "name": "Student A"},
            {"id": 4, "name": "Student B"}
        ])))
        .mount(&server)
        .await;

    let students: Vec<_> = assignment
        .get_gradeable_students()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(students.len(), 2);
}

#[tokio::test]
async fn test_assignment_override_edit() {
    let server = MockServer::start().await;
    let assignment = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignments/2/overrides/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(override_json(20, 2)))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/assignments/2/overrides/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20,
            "assignment_id": 2,
            "title": "Updated Override"
        })))
        .mount(&server)
        .await;

    let ov = assignment.get_override(20).await.unwrap();
    let updated = ov
        .edit(AssignmentOverrideParams {
            title: Some("Updated Override".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.title.as_deref(), Some("Updated Override"));
}

#[tokio::test]
async fn test_assignment_override_delete() {
    let server = MockServer::start().await;
    let assignment = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignments/2/overrides/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(override_json(20, 2)))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/assignments/2/overrides/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(override_json(20, 2)))
        .mount(&server)
        .await;

    let ov = assignment.get_override(20).await.unwrap();
    let deleted = ov.delete().await.unwrap();
    assert_eq!(deleted.id, 20);
}

#[tokio::test]
async fn test_course_get_assignment_groups() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignment_groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            group_json(5, 1),
            group_json(6, 1)
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let groups: Vec<_> = course
        .get_assignment_groups()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].id, 5);
    assert_eq!(groups[0].course_id, Some(1));
}

#[tokio::test]
async fn test_assignment_group_edit() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignment_groups"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([group_json(5, 1)])),
        )
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/assignment_groups/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "name": "Updated Group",
            "group_weight": 60.0
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let groups: Vec<_> = course
        .get_assignment_groups()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    let group = &groups[0];
    let updated = group
        .edit(AssignmentGroupParams {
            name: Some("Updated Group".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.name.as_deref(), Some("Updated Group"));
}

#[tokio::test]
async fn test_assignment_group_delete() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignment_groups"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([group_json(5, 1)])),
        )
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/assignment_groups/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(group_json(5, 1)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let groups: Vec<_> = course
        .get_assignment_groups()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    let deleted = groups[0].delete().await.unwrap();
    assert_eq!(deleted.id, 5);
}

#[tokio::test]
async fn test_assignment_set_extensions() {
    let server = MockServer::start().await;
    let assignment = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/assignments/2/extensions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "assignment_extensions": [{"user_id": 10, "extra_attempts": 2}]
        })))
        .mount(&server)
        .await;

    let result = assignment
        .set_extensions(&[(
            "assignment_extensions[][user_id]".to_string(),
            "10".to_string(),
        ), (
            "assignment_extensions[][extra_attempts]".to_string(),
            "2".to_string(),
        )])
        .await
        .unwrap();
    assert!(result.get("assignment_extensions").is_some());
}
