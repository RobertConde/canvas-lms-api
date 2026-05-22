use canvas_lms_api::{resources::folder::UpdateFolderParams, Canvas};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn folder_json(id: u64, name: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": name,
        "full_name": format!("course files/{}", name),
        "context_id": 1,
        "context_type": "Course",
        "files_count": 0,
        "folders_count": 0
    })
}

fn file_json(id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "display_name": "test.txt",
        "filename": "test.txt",
        "content_type": "text/plain",
        "size": 10,
        "url": "https://example.com/test.txt"
    })
}

#[tokio::test]
async fn test_folder_update() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/folders/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(folder_json(1, "old name")))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/folders/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(folder_json(1, "new name")))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let folder = canvas.get_folder(1).await.unwrap();
    assert_eq!(folder.name.as_deref(), Some("old name"));

    let updated = folder
        .update(UpdateFolderParams {
            name: Some("new name".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.name.as_deref(), Some("new name"));
}

#[tokio::test]
async fn test_folder_delete() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/folders/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(folder_json(1, "docs")))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/folders/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(folder_json(1, "docs")))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let folder = canvas.get_folder(1).await.unwrap();
    let deleted = folder.delete().await.unwrap();
    assert_eq!(deleted.id, 1);
}

#[tokio::test]
async fn test_folder_get_files() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/folders/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(folder_json(1, "docs")))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/folders/1/files"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([file_json(10)])),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let folder = canvas.get_folder(1).await.unwrap();
    let files: Vec<_> = folder
        .get_files()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].id, 10);
}

#[tokio::test]
async fn test_folder_get_folders() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/folders/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(folder_json(1, "root")))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/folders/1/folders"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([folder_json(2, "sub")])),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let folder = canvas.get_folder(1).await.unwrap();
    let subs: Vec<_> = folder
        .get_folders()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].id, 2);
}

#[tokio::test]
async fn test_folder_create_folder() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/folders/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(folder_json(1, "root")))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/folders/1/folders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(folder_json(2, "new sub")))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let folder = canvas.get_folder(1).await.unwrap();
    let new_folder = folder.create_folder("new sub").await.unwrap();
    assert_eq!(new_folder.name.as_deref(), Some("new sub"));
}

#[tokio::test]
async fn test_folder_copy_file() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/folders/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(folder_json(1, "dest")))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/folders/1/copy_file"))
        .respond_with(ResponseTemplate::new(200).set_body_json(file_json(99)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let folder = canvas.get_folder(1).await.unwrap();
    let copied = folder.copy_file(42).await.unwrap();
    assert_eq!(copied.id, 99);
}
