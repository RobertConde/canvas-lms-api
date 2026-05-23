use canvas_lms_api::resources::external_tool::{ExternalTool, ExternalToolParams};
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_course_tool(server: &MockServer) -> ExternalTool {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/external_tools/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "name": "Tool",
            "course_id": 1
        })))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas
        .get_course(1)
        .await
        .unwrap()
        .get_external_tool(5)
        .await
        .unwrap()
}

async fn setup_account_tool(server: &MockServer) -> ExternalTool {
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/external_tools/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "name": "Tool",
            "account_id": 1
        })))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas
        .get_account(1)
        .await
        .unwrap()
        .get_external_tool(5)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_external_tool_edit_course() {
    let server = MockServer::start().await;
    let tool = setup_course_tool(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/external_tools/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "name": "Updated Tool",
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let params = ExternalToolParams {
        name: Some("Updated Tool".to_string()),
        ..Default::default()
    };
    let updated = tool.edit(params).await.unwrap();
    assert_eq!(updated.id, 5);
    assert_eq!(updated.name.as_deref(), Some("Updated Tool"));
}

#[tokio::test]
async fn test_external_tool_delete_course() {
    let server = MockServer::start().await;
    let tool = setup_course_tool(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/external_tools/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "name": "Tool",
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let deleted = tool.delete().await.unwrap();
    assert_eq!(deleted.id, 5);
}

#[tokio::test]
async fn test_external_tool_edit_account() {
    let server = MockServer::start().await;
    let tool = setup_account_tool(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/accounts/1/external_tools/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "name": "Updated Account Tool",
            "account_id": 1
        })))
        .mount(&server)
        .await;

    let params = ExternalToolParams {
        name: Some("Updated Account Tool".to_string()),
        ..Default::default()
    };
    let updated = tool.edit(params).await.unwrap();
    assert_eq!(updated.id, 5);
    assert_eq!(updated.name.as_deref(), Some("Updated Account Tool"));
}

#[tokio::test]
async fn test_external_tool_delete_account() {
    let server = MockServer::start().await;
    let tool = setup_account_tool(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/accounts/1/external_tools/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "name": "Tool",
            "account_id": 1
        })))
        .mount(&server)
        .await;

    let deleted = tool.delete().await.unwrap();
    assert_eq!(deleted.id, 5);
}

#[tokio::test]
async fn test_external_tool_get_sessionless_launch_url_course() {
    let server = MockServer::start().await;
    let tool = setup_course_tool(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/external_tools/sessionless_launch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "5",
            "name": "Tool",
            "url": "https://lti.example.com/launch"
        })))
        .mount(&server)
        .await;

    let result = tool
        .get_sessionless_launch_url(&[("id".to_string(), "5".to_string())])
        .await
        .unwrap();
    assert_eq!(result["name"], "Tool");
    assert!(result.get("url").is_some());
}

#[tokio::test]
async fn test_external_tool_get_sessionless_launch_url_account() {
    let server = MockServer::start().await;
    let tool = setup_account_tool(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/external_tools/sessionless_launch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "5",
            "name": "Account Tool",
            "url": "https://lti.example.com/account-launch"
        })))
        .mount(&server)
        .await;

    let result = tool
        .get_sessionless_launch_url(&[("id".to_string(), "5".to_string())])
        .await
        .unwrap();
    assert_eq!(result["name"], "Account Tool");
}

#[tokio::test]
async fn test_external_tool_get_parent_course() {
    let server = MockServer::start().await;
    let tool = setup_course_tool(&server).await;
    // setup_course_tool already mounts GET /courses/1 → {"id": 1}
    let parent = tool.get_parent().await.unwrap();
    assert_eq!(parent["id"], 1);
}

#[tokio::test]
async fn test_external_tool_get_parent_account() {
    let server = MockServer::start().await;
    let tool = setup_account_tool(&server).await;
    // setup_account_tool already mounts GET /accounts/1 → {"id": 1}
    let parent = tool.get_parent().await.unwrap();
    assert_eq!(parent["id"], 1);
}
