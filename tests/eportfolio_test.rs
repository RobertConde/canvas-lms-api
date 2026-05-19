use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn portfolio_json(id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": "My Portfolio",
        "public": false,
        "user_id": 10
    })
}

async fn get_portfolio(server: &MockServer) -> canvas_lms_api::resources::eportfolio::EPortfolio {
    Mock::given(method("GET"))
        .and(path("/api/v1/eportfolios/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(portfolio_json(5)))
        .mount(server)
        .await;
    Canvas::new(&server.uri(), "token")
        .unwrap()
        .get_eportfolio(5)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_get_eportfolio() {
    let server = MockServer::start().await;
    let p = get_portfolio(&server).await;
    assert_eq!(p.id, 5);
    assert_eq!(p.name.as_deref(), Some("My Portfolio"));
}

#[tokio::test]
async fn test_eportfolio_delete() {
    let server = MockServer::start().await;
    let p = get_portfolio(&server).await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/eportfolios/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(portfolio_json(5)))
        .mount(&server)
        .await;

    let deleted = p.delete().await.unwrap();
    assert_eq!(deleted.id, 5);
}

#[tokio::test]
async fn test_eportfolio_get_pages() {
    let server = MockServer::start().await;
    let p = get_portfolio(&server).await;
    Mock::given(method("GET"))
        .and(path("/api/v1/eportfolios/5/pages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "About Me", "position": 1},
            {"id": 2, "name": "Projects", "position": 2}
        ])))
        .mount(&server)
        .await;

    let pages = p.get_pages().collect_all().await.unwrap();
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].name.as_deref(), Some("About Me"));
}

#[tokio::test]
async fn test_eportfolio_moderate() {
    let server = MockServer::start().await;
    let p = get_portfolio(&server).await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/eportfolios/5/moderate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5, "name": "My Portfolio", "spam_status": "marked_as_spam"
        })))
        .mount(&server)
        .await;

    let moderated = p.moderate("marked_as_spam").await.unwrap();
    assert_eq!(moderated.spam_status.as_deref(), Some("marked_as_spam"));
}

#[tokio::test]
async fn test_eportfolio_restore() {
    let server = MockServer::start().await;
    let p = get_portfolio(&server).await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/eportfolios/5/restore"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5, "name": "My Portfolio", "deleted_at": null
        })))
        .mount(&server)
        .await;

    let restored = p.restore().await.unwrap();
    assert_eq!(restored.id, 5);
}
