use canvas_lms_api::{resources::file::UpdateFileParams, Canvas};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn file_json(id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "display_name": "test.txt",
        "filename": "test.txt",
        "content_type": "text/plain",
        "size": 12,
        "url": "https://example.com/files/test.txt",
        "locked": false
    })
}

#[tokio::test]
async fn test_file_update() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/files/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(file_json(1)))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/files/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "display_name": "renamed.txt",
            "filename": "renamed.txt",
            "content_type": "text/plain",
            "size": 12,
            "url": "https://example.com/files/renamed.txt",
            "locked": false
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let file = canvas.get_file(1).await.unwrap();
    assert_eq!(file.display_name.as_deref(), Some("test.txt"));

    let updated = file
        .update(UpdateFileParams {
            name: Some("renamed.txt".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.display_name.as_deref(), Some("renamed.txt"));
}

#[tokio::test]
async fn test_file_delete() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/files/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(file_json(1)))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/files/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(file_json(1)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let file = canvas.get_file(1).await.unwrap();
    let deleted = file.delete().await.unwrap();
    assert_eq!(deleted.id, 1);
}

#[tokio::test]
async fn test_file_get_contents() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/files/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "display_name": "hello.txt",
            "url": format!("{}/download/hello.txt", server.uri()),
            "size": 5
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/download/hello.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"hello".to_vec()))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let file = canvas.get_file(1).await.unwrap();
    let contents = file.get_contents().await.unwrap();
    assert_eq!(contents, b"hello");
}

#[tokio::test]
async fn test_file_download() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/files/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "display_name": "hello.txt",
            "url": format!("{}/download/hello.txt", server.uri()),
            "size": 5
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/download/hello.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"hello".to_vec()))
        .mount(&server)
        .await;

    let tmp = std::env::temp_dir().join("canvas_test_file_download.txt");
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let file = canvas.get_file(1).await.unwrap();
    file.download(&tmp).await.unwrap();

    let contents = std::fs::read(&tmp).unwrap();
    assert_eq!(contents, b"hello");
    let _ = std::fs::remove_file(&tmp);
}
