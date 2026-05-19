#![cfg(feature = "blocking")]

use canvas_lms_api::CanvasBlocking;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn test_blocking_get_course() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let server = rt.block_on(MockServer::start());

    rt.block_on(
        Mock::given(method("GET"))
            .and(path("/api/v1/courses/42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 42,
                "name": "Blocking Rust",
                "course_code": "BRUST-42",
                "workflow_state": "available",
                "account_id": 1
            })))
            .mount(&server),
    );

    let canvas = CanvasBlocking::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(42).unwrap();

    assert_eq!(course.id, 42);
    assert_eq!(course.name.as_deref(), Some("Blocking Rust"));
    assert_eq!(course.course_code.as_deref(), Some("BRUST-42"));
}

#[test]
fn test_blocking_get_courses_pagination() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let server = rt.block_on(MockServer::start());

    let page2_url = format!("{}/api/v1/courses?page=2&per_page=100", server.uri());

    // First page — Link header points to page 2
    rt.block_on(
        Mock::given(method("GET"))
            .and(path("/api/v1/courses"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("Link", format!("<{}>; rel=\"next\"", page2_url))
                    .set_body_json(serde_json::json!([
                        {"id": 1, "name": "Course A"},
                        {"id": 2, "name": "Course B"}
                    ])),
            )
            .up_to_n_times(1)
            .mount(&server),
    );

    // Second page — no Link header (last page)
    rt.block_on(
        Mock::given(method("GET"))
            .and(query_param("page", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"id": 3, "name": "Course C"}
            ])))
            .mount(&server),
    );

    let canvas = CanvasBlocking::new(&server.uri(), "test-token").unwrap();
    let courses = canvas.get_courses().unwrap();

    assert_eq!(courses.len(), 3);
    assert_eq!(courses[0].id, 1);
    assert_eq!(courses[1].id, 2);
    assert_eq!(courses[2].id, 3);
}
