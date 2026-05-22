use canvas_lms_api::{resources::rubric::RubricParams, Canvas};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn make_course(server: &MockServer) -> canvas_lms_api::resources::course::Course {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "Test Course"
        })))
        .mount(server)
        .await;
    Canvas::new(&server.uri(), "test-token")
        .unwrap()
        .get_course(1)
        .await
        .unwrap()
}

async fn make_rubric(
    server: &MockServer,
    course: &canvas_lms_api::resources::course::Course,
) -> canvas_lms_api::resources::rubric::Rubric {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/rubrics/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
            "title": "Essay Rubric",
            "course_id": 1,
            "points_possible": 10.0
        })))
        .mount(server)
        .await;
    course.get_rubric(3).await.unwrap()
}

async fn make_assoc(
    server: &MockServer,
    course: &canvas_lms_api::resources::course::Course,
) -> canvas_lms_api::resources::rubric::RubricAssociation {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/rubric_associations/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "rubric_id": 3,
            "association_id": 1,
            "association_type": "Course",
            "course_id": 1,
            "purpose": "bookmarking"
        })))
        .mount(server)
        .await;
    course.get_rubric_association(7).await.unwrap()
}

// ---- Rubric instance methods ----

#[tokio::test]
async fn test_rubric_delete() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;
    let rubric = make_rubric(&server, &course).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/rubrics/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
            "title": "Essay Rubric",
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let deleted = rubric.delete().await.unwrap();
    assert_eq!(deleted.id, 3);
    assert_eq!(deleted.title.as_deref(), Some("Essay Rubric"));
}

#[tokio::test]
async fn test_rubric_update() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;
    let rubric = make_rubric(&server, &course).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/rubrics/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
            "title": "Updated Rubric",
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let params = RubricParams {
        title: Some("Updated Rubric".to_string()),
        ..Default::default()
    };
    let updated = rubric.update(params).await.unwrap();
    assert_eq!(updated.title.as_deref(), Some("Updated Rubric"));
}

// ---- RubricAssociation instance methods ----

#[tokio::test]
async fn test_rubric_association_update() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;
    let assoc = make_assoc(&server, &course).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/rubric_associations/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "rubric_id": 3,
            "association_id": 1,
            "association_type": "Course",
            "course_id": 1,
            "purpose": "grading"
        })))
        .mount(&server)
        .await;

    let updated = assoc
        .update(&[(
            "rubric_association[purpose]".to_string(),
            "grading".to_string(),
        )])
        .await
        .unwrap();
    assert_eq!(updated.id, 7);
    assert_eq!(updated.purpose.as_deref(), Some("grading"));
}

#[tokio::test]
async fn test_rubric_association_delete() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;
    let assoc = make_assoc(&server, &course).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/rubric_associations/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "rubric_id": 3,
            "association_id": 1,
            "association_type": "Course",
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let deleted = assoc.delete().await.unwrap();
    assert_eq!(deleted.id, 7);
    assert_eq!(deleted.rubric_id, Some(3));
}

#[tokio::test]
async fn test_rubric_association_create_rubric_assessment() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;
    let assoc = make_assoc(&server, &course).await;

    Mock::given(method("POST"))
        .and(path(
            "/api/v1/courses/1/rubric_associations/7/rubric_assessments",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20,
            "rubric_id": 3,
            "rubric_association_id": 7,
            "score": 8.0,
            "course_id": 1,
            "artifact_type": "Submission",
            "artifact_id": 100,
            "assessment_type": "grading"
        })))
        .mount(&server)
        .await;

    let assessment = assoc
        .create_rubric_assessment(&[
            ("rubric_assessment[user_id]".to_string(), "5".to_string()),
            (
                "rubric_assessment[assessment_type]".to_string(),
                "grading".to_string(),
            ),
        ])
        .await
        .unwrap();
    assert_eq!(assessment.id, 20);
    assert_eq!(assessment.rubric_id, Some(3));
    assert_eq!(assessment.score, Some(8.0));
}

// ---- RubricAssessment instance methods ----

async fn make_assessment(
    server: &MockServer,
    assoc: &canvas_lms_api::resources::rubric::RubricAssociation,
) -> canvas_lms_api::resources::rubric::RubricAssessment {
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/courses/1/rubric_associations/7/rubric_assessments",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20,
            "rubric_id": 3,
            "rubric_association_id": 7,
            "score": 8.0,
            "course_id": 1,
            "artifact_type": "Submission",
            "artifact_id": 100
        })))
        .mount(server)
        .await;
    assoc
        .create_rubric_assessment(&[("rubric_assessment[user_id]".to_string(), "5".to_string())])
        .await
        .unwrap()
}

#[tokio::test]
async fn test_rubric_assessment_update() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;
    let assoc = make_assoc(&server, &course).await;
    let assessment = make_assessment(&server, &assoc).await;

    Mock::given(method("PUT"))
        .and(path(
            "/api/v1/courses/1/rubric_associations/7/rubric_assessments/20",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20,
            "rubric_id": 3,
            "rubric_association_id": 7,
            "score": 9.0,
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let updated = assessment
        .update(&[("rubric_assessment[score]".to_string(), "9.0".to_string())])
        .await
        .unwrap();
    assert_eq!(updated.id, 20);
    assert_eq!(updated.score, Some(9.0));
}

#[tokio::test]
async fn test_rubric_assessment_delete() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;
    let assoc = make_assoc(&server, &course).await;
    let assessment = make_assessment(&server, &assoc).await;

    Mock::given(method("DELETE"))
        .and(path(
            "/api/v1/courses/1/rubric_associations/7/rubric_assessments/20",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20,
            "rubric_id": 3,
            "rubric_association_id": 7,
            "score": 8.0,
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let deleted = assessment.delete().await.unwrap();
    assert_eq!(deleted.id, 20);
    assert_eq!(deleted.rubric_id, Some(3));
}
