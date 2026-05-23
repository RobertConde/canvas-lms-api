use canvas_lms_api::{resources::submission::EditSubmissionParams, Canvas};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn submission_json(id: u64, course_id: u64, assignment_id: u64, user_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "course_id": course_id,
        "assignment_id": assignment_id,
        "user_id": user_id,
        "grade": "A",
        "score": 100.0,
        "workflow_state": "graded",
        "excused": false
    })
}

async fn setup_with_canvas(
    canvas: &canvas_lms_api::Canvas,
    server: &MockServer,
) -> canvas_lms_api::resources::submission::Submission {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignments/2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"id": 2, "course_id": 1, "name": "Assignment 1"}),
        ))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignments/2/submissions/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(submission_json(10, 1, 2, 3)))
        .mount(server)
        .await;

    let course = canvas.get_course(1).await.unwrap();
    let assignment = course.get_assignment(2).await.unwrap();
    assignment.get_submission(3).await.unwrap()
}

async fn setup(server: &MockServer) -> canvas_lms_api::resources::submission::Submission {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignments/2"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"id": 2, "course_id": 1, "name": "Assignment 1"}),
            ),
        )
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignments/2/submissions/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(submission_json(10, 1, 2, 3)))
        .mount(server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let assignment = course.get_assignment(2).await.unwrap();
    assignment.get_submission(3).await.unwrap()
}

#[tokio::test]
async fn test_submission_edit() {
    let server = MockServer::start().await;
    let submission = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/assignments/2/submissions/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "course_id": 1,
            "assignment_id": 2,
            "user_id": 3,
            "excused": true
        })))
        .mount(&server)
        .await;

    let updated = submission
        .edit(EditSubmissionParams {
            excuse: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.excused, Some(true));
}

#[tokio::test]
async fn test_submission_mark_read() {
    let server = MockServer::start().await;
    let submission = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/assignments/2/submissions/3/read"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    submission.mark_read().await.unwrap();
}

#[tokio::test]
async fn test_submission_mark_unread() {
    let server = MockServer::start().await;
    let submission = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/assignments/2/submissions/3/read"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    submission.mark_unread().await.unwrap();
}

#[tokio::test]
async fn test_submission_create_peer_review() {
    let server = MockServer::start().await;
    let submission = setup(&server).await;

    Mock::given(method("POST"))
        .and(path(
            "/api/v1/courses/1/assignments/2/submissions/3/peer_reviews",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "assessor_id": 99,
            "user_id": 7,
            "workflow_state": "assigned"
        })))
        .mount(&server)
        .await;

    let pr = submission.create_submission_peer_review(99).await.unwrap();
    assert_eq!(pr.assessor_id, Some(99));
    assert_eq!(pr.workflow_state.as_deref(), Some("assigned"));
}

#[tokio::test]
async fn test_submission_delete_peer_review() {
    let server = MockServer::start().await;
    let submission = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path(
            "/api/v1/courses/1/assignments/2/submissions/3/peer_reviews",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "assessor_id": 99,
            "user_id": 7,
            "workflow_state": "completed"
        })))
        .mount(&server)
        .await;

    let pr = submission.delete_submission_peer_review(99).await.unwrap();
    assert_eq!(pr.assessor_id, Some(99));
    assert_eq!(pr.workflow_state.as_deref(), Some("completed"));
}

#[tokio::test]
async fn test_submission_get_peer_reviews() {
    let server = MockServer::start().await;
    let submission = setup(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/courses/1/assignments/2/submissions/3/peer_reviews",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"assessor_id": 10, "user_id": 7, "workflow_state": "assigned"},
            {"assessor_id": 11, "user_id": 7, "workflow_state": "completed"}
        ])))
        .mount(&server)
        .await;

    let prs: Vec<_> = submission
        .get_submission_peer_reviews()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(prs.len(), 2);
    assert_eq!(prs[0].assessor_id, Some(10));
    assert_eq!(prs[1].workflow_state.as_deref(), Some("completed"));
}

// ============================================================================
// v0.8.0 Batch 4 — PeerReview + upload_comment
// ============================================================================

#[tokio::test]
async fn test_submission_upload_comment() {
    use canvas_lms_api::upload::UploadRequest;

    let canvas_server = MockServer::start().await;
    let upload_server = MockServer::start().await;

    // Setup mock for get_course and get submission path
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1, "name": "Test Course"
        })))
        .mount(&canvas_server)
        .await;

    // Step 1: Canvas returns upload intent for comment file
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/courses/1/assignments/2/submissions/3/comments/files",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "upload_url": format!("{}/s3-upload", upload_server.uri()),
            "upload_params": {
                "key": "comments/attachment",
                "Policy": "FAKEPOLICY"
            }
        })))
        .mount(&canvas_server)
        .await;

    // Step 2: Upload target returns File object
    Mock::given(method("POST"))
        .and(path("/s3-upload"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 77,
            "display_name": "feedback.pdf",
            "filename": "feedback.pdf",
            "content_type": "application/pdf",
            "size": 1024
        })))
        .mount(&upload_server)
        .await;

    let canvas = canvas_lms_api::Canvas::new(&canvas_server.uri(), "test-token").unwrap();
    let submission = setup_with_canvas(&canvas, &canvas_server).await;

    let request = UploadRequest {
        name: "feedback.pdf".to_string(),
        size: 1024,
        content_type: Some("application/pdf".to_string()),
        ..Default::default()
    };

    let file = submission
        .upload_comment(request, vec![0u8; 1024])
        .await
        .unwrap();
    assert_eq!(file.id, 77);
    assert_eq!(file.display_name.as_deref(), Some("feedback.pdf"));
}
