#![cfg(feature = "graphql")]

use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_graphql_query_no_variables() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "allCourses": [
                    {"id": "1", "name": "Intro to Rust"}
                ]
            }
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let gql = canvas.graphql();
    let result = gql.query("{ allCourses { id name } }", None).await.unwrap();

    let courses = &result["data"]["allCourses"];
    assert!(courses.is_array());
    assert_eq!(courses[0]["name"], "Intro to Rust");
}

#[tokio::test]
async fn test_graphql_query_with_variables() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "course": {"id": "42", "name": "Advanced Topics"}
            }
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let gql = canvas.graphql();
    let variables = serde_json::json!({"courseId": "42"});
    let result = gql
        .query(
            "query GetCourse($courseId: ID!) { course(id: $courseId) { id name } }",
            Some(variables),
        )
        .await
        .unwrap();

    assert_eq!(result["data"]["course"]["name"], "Advanced Topics");
}

#[tokio::test]
async fn test_graphql_errors_propagated() {
    let server = MockServer::start().await;

    // Canvas returns 400 for bad GraphQL queries
    Mock::given(method("POST"))
        .and(path("/api/graphql"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "errors": [{"message": "Parse error on \"{\", expected: ID"}]
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let gql = canvas.graphql();
    let err = gql.query("{ bad query }", None).await;
    assert!(err.is_err());
}
