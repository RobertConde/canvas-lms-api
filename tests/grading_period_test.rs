use canvas_lms_api::resources::grading_period::GradingPeriodParams;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn gp_json(id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "title": "Q1",
        "start_date": "2026-01-01",
        "end_date": "2026-03-31",
        "close_date": "2026-04-07",
        "weight": 25.0
    })
}

async fn make_course(server: &MockServer) -> canvas_lms_api::resources::course::Course {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Canvas::new(&server.uri(), "token")
        .unwrap()
        .get_course(1)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_get_grading_periods() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/grading_periods"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([gp_json(1), gp_json(2)])),
        )
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let periods = course.get_grading_periods().collect_all().await.unwrap();
    assert_eq!(periods.len(), 2);
    assert_eq!(periods[0].title.as_deref(), Some("Q1"));
    assert_eq!(periods[0].course_id, Some(1));
}

#[tokio::test]
async fn test_grading_period_update() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/grading_periods"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([gp_json(5)])))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/grading_periods/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "grading_periods": [{
                "id": 5, "title": "Q1 Updated", "start_date": "2026-01-01",
                "end_date": "2026-03-31"
            }]
        })))
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let mut periods = course.get_grading_periods().collect_all().await.unwrap();
    let gp = periods.remove(0);
    let updated = gp
        .update(GradingPeriodParams {
            start_date: "2026-01-01".into(),
            end_date: "2026-03-31".into(),
            title: Some("Q1 Updated".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.title.as_deref(), Some("Q1 Updated"));
}

#[tokio::test]
async fn test_grading_period_delete() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/grading_periods"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([gp_json(5)])))
        .mount(&server)
        .await;
    // Canvas returns 204 No Content; wiremock can return an empty body that serde parses as null
    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/grading_periods/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(null)))
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let mut periods = course.get_grading_periods().collect_all().await.unwrap();
    let gp = periods.remove(0);
    gp.delete().await.unwrap();
}
