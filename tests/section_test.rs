use canvas_lms_api::{
    resources::section::{EnrollUserParams, UpdateSectionParams},
    Canvas,
};
use futures::StreamExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn section_json(id: u64, course_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "course_id": course_id,
        "name": "Section A",
        "sis_section_id": null,
        "total_students": 5
    })
}

fn enrollment_json(id: u64, course_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "course_id": course_id,
        "user_id": 10,
        "type": "StudentEnrollment",
        "enrollment_state": "active"
    })
}

async fn setup(server: &MockServer) -> canvas_lms_api::resources::section::Section {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/sections/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(section_json(5, 1)))
        .mount(server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    course.get_section(5).await.unwrap()
}

#[tokio::test]
async fn test_section_edit() {
    let server = MockServer::start().await;
    let section = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/sections/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "course_id": 1,
            "name": "Section B"
        })))
        .mount(&server)
        .await;

    let updated = section
        .edit(UpdateSectionParams {
            name: Some("Section B".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.name.as_deref(), Some("Section B"));
}

#[tokio::test]
async fn test_section_delete() {
    let server = MockServer::start().await;
    let section = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/sections/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(section_json(5, 1)))
        .mount(&server)
        .await;

    let deleted = section.delete().await.unwrap();
    assert_eq!(deleted.id, 5);
}

#[tokio::test]
async fn test_section_enroll_user() {
    let server = MockServer::start().await;
    let section = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/sections/5/enrollments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(enrollment_json(20, 1)))
        .mount(&server)
        .await;

    let enrollment = section
        .enroll_user(EnrollUserParams {
            user_id: 10,
            r#type: Some("StudentEnrollment".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(enrollment.id, 20);
    assert_eq!(enrollment.user_id, Some(10));
}

#[tokio::test]
async fn test_section_get_enrollments() {
    let server = MockServer::start().await;
    let section = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/sections/5/enrollments"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([enrollment_json(20, 1), enrollment_json(21, 1)])),
        )
        .mount(&server)
        .await;

    let enrollments: Vec<_> = section
        .get_enrollments()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(enrollments.len(), 2);
    assert_eq!(enrollments[0].id, 20);
}

#[tokio::test]
async fn test_section_cross_list_section() {
    let server = MockServer::start().await;
    let section = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/sections/5/crosslist/2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(section_json(5, 2)))
        .mount(&server)
        .await;

    let moved = section.cross_list_section(2).await.unwrap();
    assert_eq!(moved.course_id, Some(2));
}

#[tokio::test]
async fn test_section_decross_list_section() {
    let server = MockServer::start().await;
    let section = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/sections/5/crosslist"))
        .respond_with(ResponseTemplate::new(200).set_body_json(section_json(5, 1)))
        .mount(&server)
        .await;

    let restored = section.decross_list_section().await.unwrap();
    assert_eq!(restored.course_id, Some(1));
}

#[tokio::test]
async fn test_section_get_assignment_override() {
    let server = MockServer::start().await;
    let section = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/sections/5/assignments/10/override"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 100,
            "assignment_id": 10,
            "course_section_id": 5
        })))
        .mount(&server)
        .await;

    let override_val = section.get_assignment_override(10).await.unwrap();
    assert_eq!(override_val["assignment_id"], 10);
}

#[tokio::test]
async fn test_section_get_multiple_submissions() {
    let server = MockServer::start().await;
    let section = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/sections/5/students/submissions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 30, "assignment_id": 10, "user_id": 5, "grade": "A"}
        ])))
        .mount(&server)
        .await;

    let subs: Vec<_> = section
        .get_multiple_submissions()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].id, 30);
}

#[tokio::test]
async fn test_section_submissions_bulk_update() {
    let server = MockServer::start().await;
    let section = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/sections/5/submissions/update_grades"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99,
            "workflow_state": "queued",
            "completion": 0
        })))
        .mount(&server)
        .await;

    let progress = section
        .submissions_bulk_update(&[(
            "grade_data[5][10][posted_grade]".to_string(),
            "A".to_string(),
        )])
        .await
        .unwrap();
    assert_eq!(progress.id, 99);
    assert_eq!(progress.workflow_state.as_deref(), Some("queued"));
}
