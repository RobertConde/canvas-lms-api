use canvas_lms_api::{
    resources::module::{CreateModuleItemParams, UpdateModuleItemParams, UpdateModuleParams},
    Canvas,
};
use futures::StreamExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn module_json(id: u64, course_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "course_id": course_id,
        "name": "Module 1",
        "position": 1,
        "published": true
    })
}

fn item_json(id: u64, module_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "module_id": module_id,
        "title": "Item 1",
        "type": "Assignment",
        "content_id": 10,
        "position": 1,
        "completion_requirement": {
            "type": "must_view",
            "completed": false
        }
    })
}

async fn setup(server: &MockServer) -> canvas_lms_api::resources::module::Module {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/modules/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(module_json(1, 1)))
        .mount(server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    course.get_module(1).await.unwrap()
}

async fn setup_with_item(
    server: &MockServer,
) -> canvas_lms_api::resources::module::ModuleItem {
    let module = setup(server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/modules/1/items/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item_json(5, 1)))
        .mount(server)
        .await;

    module.get_module_item(5).await.unwrap()
}

#[tokio::test]
async fn test_module_edit() {
    let server = MockServer::start().await;
    let module = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/modules/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "course_id": 1,
            "name": "New Name"
        })))
        .mount(&server)
        .await;

    let updated = module
        .edit(UpdateModuleParams {
            name: Some("New Name".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.name.as_deref(), Some("New Name"));
    assert_eq!(updated.course_id, Some(1));
}

#[tokio::test]
async fn test_module_delete() {
    let server = MockServer::start().await;
    let module = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/modules/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(module_json(1, 1)))
        .mount(&server)
        .await;

    let deleted = module.delete().await.unwrap();
    assert_eq!(deleted.id, 1);
    assert_eq!(deleted.course_id, Some(1));
}

#[tokio::test]
async fn test_module_relock() {
    let server = MockServer::start().await;
    let module = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/modules/1/relock"))
        .respond_with(ResponseTemplate::new(200).set_body_json(module_json(1, 1)))
        .mount(&server)
        .await;

    let relocked = module.relock().await.unwrap();
    assert_eq!(relocked.id, 1);
    assert_eq!(relocked.course_id, Some(1));
}

#[tokio::test]
async fn test_module_get_module_items() {
    let server = MockServer::start().await;
    let module = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/modules/1/items"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([item_json(5, 1), item_json(6, 1)])),
        )
        .mount(&server)
        .await;

    let items: Vec<_> = module
        .get_module_items()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].id, 5);
    assert_eq!(items[0].course_id, Some(1));
    assert_eq!(items[0].module_id, Some(1));
}

#[tokio::test]
async fn test_module_get_module_item() {
    let server = MockServer::start().await;
    let module = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/modules/1/items/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item_json(5, 1)))
        .mount(&server)
        .await;

    let item = module.get_module_item(5).await.unwrap();
    assert_eq!(item.id, 5);
    assert_eq!(item.course_id, Some(1));
    assert_eq!(item.module_id, Some(1));
}

#[tokio::test]
async fn test_module_create_module_item() {
    let server = MockServer::start().await;
    let module = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/modules/1/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item_json(7, 1)))
        .mount(&server)
        .await;

    let item = module
        .create_module_item(CreateModuleItemParams {
            item_type: "Assignment".to_string(),
            content_id: Some(10),
            position: None,
            indent: None,
            page_url: None,
            external_url: None,
            new_tab: None,
            published: None,
        })
        .await
        .unwrap();
    assert_eq!(item.id, 7);
    assert_eq!(item.course_id, Some(1));
}

#[tokio::test]
async fn test_module_create_module_item_subheader() {
    let server = MockServer::start().await;
    let module = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/modules/1/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 8,
            "module_id": 1,
            "title": "Section Header",
            "type": "SubHeader"
        })))
        .mount(&server)
        .await;

    // SubHeader doesn't require content_id
    let item = module
        .create_module_item(CreateModuleItemParams {
            item_type: "SubHeader".to_string(),
            content_id: None,
            position: None,
            indent: None,
            page_url: None,
            external_url: None,
            new_tab: None,
            published: None,
        })
        .await
        .unwrap();
    assert_eq!(item.id, 8);
}

#[tokio::test]
async fn test_module_create_module_item_missing_content_id() {
    let server = MockServer::start().await;
    let module = setup(&server).await;

    // Assignment requires content_id — should fail validation
    let result = module
        .create_module_item(CreateModuleItemParams {
            item_type: "Assignment".to_string(),
            content_id: None,
            position: None,
            indent: None,
            page_url: None,
            external_url: None,
            new_tab: None,
            published: None,
        })
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_module_item_edit() {
    let server = MockServer::start().await;
    let item = setup_with_item(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/modules/1/items/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "module_id": 1,
            "title": "New Title",
            "type": "Assignment"
        })))
        .mount(&server)
        .await;

    let updated = item
        .edit(UpdateModuleItemParams {
            title: Some("New Title".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.title.as_deref(), Some("New Title"));
    assert_eq!(updated.course_id, Some(1));
    assert_eq!(updated.module_id, Some(1));
}

#[tokio::test]
async fn test_module_item_delete() {
    let server = MockServer::start().await;
    let item = setup_with_item(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/modules/1/items/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item_json(5, 1)))
        .mount(&server)
        .await;

    let deleted = item.delete().await.unwrap();
    assert_eq!(deleted.id, 5);
    assert_eq!(deleted.course_id, Some(1));
}

#[tokio::test]
async fn test_module_item_complete() {
    let server = MockServer::start().await;
    let item = setup_with_item(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/modules/1/items/5/done"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "module_id": 1,
            "title": "Item 1",
            "type": "Assignment",
            "completion_requirement": {
                "type": "must_view",
                "completed": true
            }
        })))
        .mount(&server)
        .await;

    let completed = item.complete().await.unwrap();
    assert_eq!(completed.id, 5);
    assert_eq!(completed.course_id, Some(1));
    assert!(completed
        .completion_requirement
        .as_ref()
        .map(|r| r.completed)
        .is_some());
}

#[tokio::test]
async fn test_module_item_uncomplete() {
    let server = MockServer::start().await;
    let item = setup_with_item(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/modules/1/items/5/done"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "module_id": 1,
            "title": "Item 1",
            "type": "Assignment",
            "completion_requirement": {
                "type": "must_view",
                "completed": false
            }
        })))
        .mount(&server)
        .await;

    let uncompleted = item.uncomplete().await.unwrap();
    assert_eq!(uncompleted.id, 5);
    assert_eq!(uncompleted.course_id, Some(1));
}

#[tokio::test]
async fn test_course_create_module() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/modules"))
        .respond_with(ResponseTemplate::new(200).set_body_json(module_json(10, 1)))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let module = course
        .create_module(UpdateModuleParams {
            name: Some("Module 1".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(module.id, 10);
    assert_eq!(module.course_id, Some(1));
}
