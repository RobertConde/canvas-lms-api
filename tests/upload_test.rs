use canvas_lms_api::{upload::UploadRequest, Canvas};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_upload_file() {
    // Server 1: Canvas API
    let canvas_server = MockServer::start().await;
    // Server 2: acts as S3 upload target
    let upload_server = MockServer::start().await;

    // Step 1: Canvas returns upload intent
    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "upload_url": format!("{}/s3-upload", upload_server.uri()),
            "upload_params": {
                "key": "path/to/file",
                "AWSAccessKeyId": "FAKEID"
            }
        })))
        .mount(&canvas_server)
        .await;

    // Step 2: Upload target returns File object
    Mock::given(method("POST"))
        .and(path("/s3-upload"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "display_name": "test.txt",
            "filename": "test.txt",
            "content_type": "text/plain",
            "size": 13,
            "url": "https://canvas.example.edu/files/42/download"
        })))
        .mount(&upload_server)
        .await;

    let canvas = Canvas::new(&canvas_server.uri(), "test-token").unwrap();
    let course = {
        // We need a Course with requester. Mock get_course too.
        Mock::given(method("GET"))
            .and(path("/api/v1/courses/1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 1, "name": "Test Course"
            })))
            .mount(&canvas_server)
            .await;
        canvas.get_course(1).await.unwrap()
    };

    let request = UploadRequest {
        name: "test.txt".to_string(),
        size: 13,
        content_type: Some("text/plain".to_string()),
        ..Default::default()
    };

    let file = course
        .upload_file(request, b"Hello, Canvas!".to_vec())
        .await
        .unwrap();
    assert_eq!(file.id, 42);
    assert_eq!(file.display_name.as_deref(), Some("test.txt"));
}

#[tokio::test]
async fn test_upload_strips_while1_prefix() {
    let canvas_server = MockServer::start().await;
    let upload_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "upload_url": format!("{}/upload", upload_server.uri()),
            "upload_params": {}
        })))
        .mount(&canvas_server)
        .await;

    // Some Canvas instances prefix JSON with while(1);
    Mock::given(method("POST"))
        .and(path("/upload"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(
                    r#"while(1);{"id":99,"display_name":"doc.pdf","filename":"doc.pdf","size":1024}"#,
                )
                .insert_header("content-type", "application/json"),
        )
        .mount(&upload_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&canvas_server)
        .await;

    let canvas = Canvas::new(&canvas_server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();

    let request = UploadRequest {
        name: "doc.pdf".to_string(),
        size: 1024,
        ..Default::default()
    };

    let file = course
        .upload_file(request, vec![0u8; 1024])
        .await
        .unwrap();
    assert_eq!(file.id, 99);
}
