use canvas_lms_api::{
    resources::user::{EditUserParams, UserId},
    Canvas,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn user_json(id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": "Test User",
        "login_id": "test@example.com"
    })
}

async fn setup(server: &MockServer) -> canvas_lms_api::resources::user::User {
    Mock::given(method("GET"))
        .and(path("/api/v1/users/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(user_json(42)))
        .mount(server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas.get_user(UserId::Id(42)).await.unwrap()
}

#[tokio::test]
async fn test_user_edit() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/users/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "name": "Updated Name"
        })))
        .mount(&server)
        .await;

    let updated = user
        .edit(EditUserParams {
            name: Some("Updated Name".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.name.as_deref(), Some("Updated Name"));
}

#[tokio::test]
async fn test_user_get_profile() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/profile"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "name": "Test User",
            "primary_email": "test@example.com"
        })))
        .mount(&server)
        .await;

    let profile = user.get_profile().await.unwrap();
    assert_eq!(profile["id"], 42);
}

#[tokio::test]
async fn test_user_terminate_sessions() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/users/42/sessions"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    user.terminate_sessions().await.unwrap();
}

#[tokio::test]
async fn test_user_merge_into() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/users/42/merge_into/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(user_json(42)))
        .mount(&server)
        .await;

    let merged = user.merge_into(99).await.unwrap();
    assert_eq!(merged.id, 42);
}

#[tokio::test]
async fn test_user_get_avatars() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/avatars"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"type": "gravatar", "url": "https://gravatar.com/avatar/abc"},
            {"type": "attachment", "url": "https://example.com/avatar.png"}
        ])))
        .mount(&server)
        .await;

    let avatars: Vec<_> = user.get_avatars().collect_all().await.unwrap();
    assert_eq!(avatars.len(), 2);
}

#[tokio::test]
async fn test_user_get_page_views() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/page_views"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"url": "/courses/1", "created_at": "2024-01-01T00:00:00Z"},
            {"url": "/courses/2", "created_at": "2024-01-02T00:00:00Z"}
        ])))
        .mount(&server)
        .await;

    let views: Vec<_> = user.get_page_views().collect_all().await.unwrap();
    assert_eq!(views.len(), 2);
}

#[tokio::test]
async fn test_user_get_observees() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/observees"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([user_json(10), user_json(11)])),
        )
        .mount(&server)
        .await;

    let observees: Vec<_> = user.get_observees().collect_all().await.unwrap();
    assert_eq!(observees.len(), 2);
}

#[tokio::test]
async fn test_user_add_observee() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/users/42/observees/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(user_json(10)))
        .mount(&server)
        .await;

    let observee = user.add_observee(10).await.unwrap();
    assert_eq!(observee.id, 10);
}

#[tokio::test]
async fn test_user_remove_observee() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/users/42/observees/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(user_json(10)))
        .mount(&server)
        .await;

    let removed = user.remove_observee(10).await.unwrap();
    assert_eq!(removed.id, 10);
}

#[tokio::test]
async fn test_user_show_observee() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/observees/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(user_json(10)))
        .mount(&server)
        .await;

    let observee = user.show_observee(10).await.unwrap();
    assert_eq!(observee.id, 10);
}

#[tokio::test]
async fn test_user_get_observers() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/observers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([user_json(20)])))
        .mount(&server)
        .await;

    let observers: Vec<_> = user.get_observers().collect_all().await.unwrap();
    assert_eq!(observers.len(), 1);
    assert_eq!(observers[0].id, 20);
}

#[tokio::test]
async fn test_user_get_colors() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/colors"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "custom_colors": {
                "course_1": "#E66000",
                "course_2": "#008EE2"
            }
        })))
        .mount(&server)
        .await;

    let colors = user.get_colors().await.unwrap();
    assert!(colors.get("custom_colors").is_some());
}

#[tokio::test]
async fn test_user_get_color() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/colors/course_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "hexcode": "#E66000"
        })))
        .mount(&server)
        .await;

    let color = user.get_color("course_1").await.unwrap();
    assert_eq!(color["hexcode"], "#E66000");
}

#[tokio::test]
async fn test_user_update_color() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/users/42/colors/course_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "hexcode": "#FF0000"
        })))
        .mount(&server)
        .await;

    let result = user.update_color("course_1", "#FF0000").await.unwrap();
    assert_eq!(result["hexcode"], "#FF0000");
}

#[tokio::test]
async fn test_user_get_missing_submissions() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/missing_submissions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 5, "name": "Late Assignment"}
        ])))
        .mount(&server)
        .await;

    let missing: Vec<_> = user.get_missing_submissions().collect_all().await.unwrap();
    assert_eq!(missing.len(), 1);
}

#[tokio::test]
async fn test_user_get_files() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 100, "display_name": "document.pdf", "size": 1024,
             "url": "https://example.com/files/100", "content-type": "application/pdf"}
        ])))
        .mount(&server)
        .await;

    let files: Vec<_> = user.get_files().collect_all().await.unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].id, 100);
}

#[tokio::test]
async fn test_user_get_folders() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/folders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 200, "name": "My Documents", "full_name": "My Documents",
             "context_type": "User", "context_id": 42}
        ])))
        .mount(&server)
        .await;

    let folders: Vec<_> = user.get_folders().collect_all().await.unwrap();
    assert_eq!(folders.len(), 1);
    assert_eq!(folders[0].id, 200);
}

#[tokio::test]
async fn test_user_create_folder() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/users/42/folders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 201,
            "name": "New Folder",
            "full_name": "New Folder"
        })))
        .mount(&server)
        .await;

    let folder = user.create_folder("New Folder").await.unwrap();
    assert_eq!(folder.id, 201);
    assert_eq!(folder.name.as_deref(), Some("New Folder"));
}

#[tokio::test]
async fn test_user_get_file_quota() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/files/quota"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "quota": 524288000,
            "quota_used": 1024
        })))
        .mount(&server)
        .await;

    let quota = user.get_file_quota().await.unwrap();
    assert_eq!(quota["quota"], 524288000);
}

#[tokio::test]
async fn test_user_get_user_logins() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/logins"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "unique_id": "test@example.com", "user_id": 42}
        ])))
        .mount(&server)
        .await;

    let logins: Vec<_> = user.get_user_logins().collect_all().await.unwrap();
    assert_eq!(logins.len(), 1);
}

#[tokio::test]
async fn test_user_get_settings() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/settings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "manual_mark_as_read": false,
            "release_notes_badge_disabled": false
        })))
        .mount(&server)
        .await;

    let settings = user.get_settings().await.unwrap();
    assert_eq!(settings["manual_mark_as_read"], false);
}

#[tokio::test]
async fn test_user_update_settings() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/users/42/settings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "manual_mark_as_read": true
        })))
        .mount(&server)
        .await;

    let settings = user
        .update_settings(&[("manual_mark_as_read".to_string(), "true".to_string())])
        .await
        .unwrap();
    assert_eq!(settings["manual_mark_as_read"], true);
}

#[tokio::test]
async fn test_user_create_pairing_code() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/users/42/observer_pairing_codes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": "ABC123",
            "expires_at": "2026-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    let code = user.create_pairing_code().await.unwrap();
    assert_eq!(code.code.as_deref(), Some("ABC123"));
}

#[tokio::test]
async fn test_user_get_authentication_events() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/audit/authentication/users/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "event_type": "login", "pseudonym_id": 10}
        ])))
        .mount(&server)
        .await;

    let events: Vec<_> = user
        .get_authentication_events()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type.as_deref(), Some("login"));
}

#[tokio::test]
async fn test_user_get_features() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/features"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"feature": "some_feature", "display_name": "Some Feature"}
        ])))
        .mount(&server)
        .await;

    let features: Vec<_> = user.get_features().collect_all().await.unwrap();
    assert_eq!(features.len(), 1);
    assert_eq!(features[0]["feature"], "some_feature");
}

#[tokio::test]
async fn test_user_get_enabled_features() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/features/enabled"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!(["feature_a", "feature_b"])),
        )
        .mount(&server)
        .await;

    let enabled = user.get_enabled_features().await.unwrap();
    assert_eq!(enabled.len(), 2);
    assert_eq!(enabled[0], "feature_a");
}

#[tokio::test]
async fn test_user_export_content() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/users/42/content_exports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "export_type": "common_cartridge",
            "workflow_state": "created"
        })))
        .mount(&server)
        .await;

    let export = user.export_content("common_cartridge").await.unwrap();
    assert_eq!(export["export_type"], "common_cartridge");
}

#[tokio::test]
async fn test_user_get_content_exports() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/content_exports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "export_type": "common_cartridge", "workflow_state": "exported"}
        ])))
        .mount(&server)
        .await;

    let exports: Vec<_> = user.get_content_exports().collect_all().await.unwrap();
    assert_eq!(exports.len(), 1);
    assert_eq!(exports[0]["export_type"], "common_cartridge");
}

#[tokio::test]
async fn test_user_get_eportfolios() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/eportfolios"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "My Portfolio", "public": false}
        ])))
        .mount(&server)
        .await;

    let portfolios: Vec<_> = user.get_eportfolios().collect_all().await.unwrap();
    assert_eq!(portfolios.len(), 1);
    assert_eq!(portfolios[0]["name"], "My Portfolio");
}

#[tokio::test]
async fn test_user_get_open_poll_sessions() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/poll_sessions/opened"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "poll_id": 5, "is_published": true}
        ])))
        .mount(&server)
        .await;

    let sessions: Vec<_> = user.get_open_poll_sessions().collect_all().await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0]["poll_id"], 5);
}

#[tokio::test]
async fn test_user_get_closed_poll_sessions() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/poll_sessions/closed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 2, "poll_id": 5, "is_published": false}
        ])))
        .mount(&server)
        .await;

    let sessions: Vec<_> = user.get_closed_poll_sessions().collect_all().await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0]["poll_id"], 5);
}

#[tokio::test]
async fn test_user_get_file() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/files/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "display_name": "User_File.docx",
            "filename": "User_File.docx",
            "size": 1024
        })))
        .mount(&server)
        .await;

    let file = user.get_file(1).await.unwrap();
    assert_eq!(file.id, 1);
    assert_eq!(file.display_name.as_deref(), Some("User_File.docx"));
    assert_eq!(file.size, Some(1024));
}

#[tokio::test]
async fn test_user_get_folder() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/folders/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "Folder 1"
        })))
        .mount(&server)
        .await;

    let folder = user.get_folder(1).await.unwrap();
    assert_eq!(folder.id, 1);
    assert_eq!(folder.name.as_deref(), Some("Folder 1"));
}

#[tokio::test]
async fn test_user_resolve_path() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/users/42/folders/by_path/Folder_Level_1/Folder_Level_2/Folder_Level_3",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "my_files"},
            {"id": 2, "name": "Folder_Level_1"},
            {"id": 3, "name": "Folder_Level_2"},
            {"id": 4, "name": "Folder_Level_3"}
        ])))
        .mount(&server)
        .await;

    let folders = user
        .resolve_path(Some("Folder_Level_1/Folder_Level_2/Folder_Level_3"))
        .collect_all()
        .await
        .unwrap();
    assert_eq!(folders.len(), 4);
    assert_eq!(folders[0].name.as_deref(), Some("my_files"));
    assert_eq!(folders[3].name.as_deref(), Some("Folder_Level_3"));
}

#[tokio::test]
async fn test_user_resolve_path_null() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/folders/by_path"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "my_files"}
        ])))
        .mount(&server)
        .await;

    let folders = user.resolve_path(None).collect_all().await.unwrap();
    assert_eq!(folders.len(), 1);
    assert_eq!(folders[0].name.as_deref(), Some("my_files"));
}

#[tokio::test]
async fn test_user_get_grade_change_events_for_grader() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/audit/grade_change/graders/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "uuid-1", "event_type": "grade_change", "links": {"course": 42}},
            {"id": "uuid-2", "event_type": "grade_change", "links": {"course": 42}}
        ])))
        .mount(&server)
        .await;

    let events = user
        .get_grade_change_events_for_grader()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["event_type"], "grade_change");
}

#[tokio::test]
async fn test_user_get_grade_change_events_for_student() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/audit/grade_change/students/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "uuid-1", "event_type": "grade_change", "links": {"course": 42}},
            {"id": "uuid-2", "event_type": "grade_change", "links": {"course": 42}}
        ])))
        .mount(&server)
        .await;

    let events = user
        .get_grade_change_events_for_student()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["event_type"], "grade_change");
}

#[tokio::test]
async fn test_user_get_content_migration() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/content_migrations/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "migration_type": "dummy_importer",
            "user_id": 42
        })))
        .mount(&server)
        .await;

    let migration = user.get_content_migration(1).await.unwrap();
    assert_eq!(migration.id, 1);
    assert_eq!(migration.migration_type.as_deref(), Some("dummy_importer"));
}

#[tokio::test]
async fn test_user_get_content_migrations() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/content_migrations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "migration_type": "dummy_importer", "user_id": 42},
            {"id": 2, "migration_type": "dummy_importer", "user_id": 42}
        ])))
        .mount(&server)
        .await;

    let migrations = user.get_content_migrations().collect_all().await.unwrap();
    assert_eq!(migrations.len(), 2);
    assert_eq!(migrations[0].id, 1);
    assert_eq!(
        migrations[0].migration_type.as_deref(),
        Some("dummy_importer")
    );
    assert_eq!(migrations[1].id, 2);
}

#[tokio::test]
async fn test_user_create_content_migration() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/users/42/content_migrations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
            "migration_type": "dummy_importer",
            "user_id": 42
        })))
        .mount(&server)
        .await;

    let migration = user
        .create_content_migration("dummy_importer")
        .await
        .unwrap();
    assert_eq!(migration.id, 3);
    assert_eq!(migration.migration_type.as_deref(), Some("dummy_importer"));
    assert_eq!(migration.user_id, Some(42));
}

#[tokio::test]
async fn test_user_get_migration_systems() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/content_migrations/migrators"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "type": "dummy_importer",
                "requires_file_upload": true,
                "name": "Dummy Importer 01"
            },
            {
                "type": "dummy_importer_02",
                "requires_file_upload": false,
                "name": "Dummy Importer 02"
            }
        ])))
        .mount(&server)
        .await;

    let migrators = user.get_migration_systems().collect_all().await.unwrap();
    assert_eq!(migrators.len(), 2);
    assert_eq!(migrators[0].r#type.as_deref(), Some("dummy_importer"));
    assert_eq!(migrators[0].requires_file_upload, Some(true));
    assert_eq!(migrators[0].name.as_deref(), Some("Dummy Importer 01"));
    assert_eq!(migrators[1].r#type.as_deref(), Some("dummy_importer_02"));
    assert_eq!(migrators[1].requires_file_upload, Some(false));
}

#[tokio::test]
async fn test_user_get_feature_flag() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/features/flags/high_contrast"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "feature": "high_contrast",
            "state": "allowed"
        })))
        .mount(&server)
        .await;

    let flag = user.get_feature_flag("high_contrast").await.unwrap();
    assert_eq!(flag["feature"], "high_contrast");
    assert_eq!(flag["state"], "allowed");
}

// ---- Batch 4: remaining depth ----

#[tokio::test]
async fn test_user_add_observee_with_credentials() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/users/42/observees"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "name": "User 5"
        })))
        .mount(&server)
        .await;

    let observee = user
        .add_observee_with_credentials(&[
            (
                "observee[unique_id]".to_string(),
                "user5@example.com".to_string(),
            ),
            ("observee[password]".to_string(), "password".to_string()),
        ])
        .await
        .unwrap();
    assert_eq!(observee.id, 5);
    assert_eq!(observee.name.as_deref(), Some("User 5"));
}

#[tokio::test]
async fn test_user_get_calendar_events() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/calendar_events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 123, "title": "Test Event", "context_code": "course_321"},
            {"id": 124, "title": "Another Event", "context_code": "course_321"}
        ])))
        .mount(&server)
        .await;

    let events = user.get_calendar_events().collect_all().await.unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["id"], 123);
    assert_eq!(events[0]["title"], "Test Event");
}

#[tokio::test]
async fn test_user_get_content_export() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/content_exports/11"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 11,
            "user_id": 1,
            "export_type": "common_cartridge",
            "workflow_state": "exported"
        })))
        .mount(&server)
        .await;

    let e = user.get_content_export(11).await.unwrap();
    assert_eq!(e["id"], 11);
    assert_eq!(e["export_type"], "common_cartridge");
}

#[tokio::test]
async fn test_user_get_licenses() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/content_licenses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "private", "name": "Private (Copyrighted)", "url": "https://copyright.gov"},
            {"id": "public_domain", "name": "Public Domain", "url": "https://creativecommons.org"}
        ])))
        .mount(&server)
        .await;

    let licenses = user.get_licenses().collect_all().await.unwrap();
    assert_eq!(licenses.len(), 2);
    assert_eq!(licenses[0].id.as_deref(), Some("private"));
}

#[tokio::test]
async fn test_user_set_usage_rights() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/users/42/usage_rights"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "use_justification": "fair_use",
            "license": "private",
            "message": "2 files updated",
            "file_ids": [1, 2]
        })))
        .mount(&server)
        .await;

    let result = user
        .set_usage_rights(&[
            ("file_ids[]".to_string(), "1".to_string()),
            (
                "usage_rights[use_justification]".to_string(),
                "fair_use".to_string(),
            ),
        ])
        .await
        .unwrap();
    assert_eq!(result.use_justification.as_deref(), Some("fair_use"));
    assert_eq!(result.message.as_deref(), Some("2 files updated"));
}

#[tokio::test]
async fn test_user_remove_usage_rights() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/users/42/usage_rights"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "2 files updated",
            "file_ids": [1, 2]
        })))
        .mount(&server)
        .await;

    let result = user
        .remove_usage_rights(&[
            ("file_ids[]".to_string(), "1".to_string()),
            ("file_ids[]".to_string(), "2".to_string()),
        ])
        .await
        .unwrap();
    assert_eq!(result.message.as_deref(), Some("2 files updated"));
}

#[tokio::test]
async fn test_user_get_assignments() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/courses/10/assignments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "course_id": 10, "name": "Assignment A"},
            {"id": 2, "course_id": 10, "name": "Assignment B"}
        ])))
        .mount(&server)
        .await;

    let assignments: Vec<_> = user.get_assignments(10).collect_all().await.unwrap();
    assert_eq!(assignments.len(), 2);
    assert_eq!(assignments[0].id, 1);
    assert_eq!(assignments[1].id, 2);
}

#[tokio::test]
async fn test_user_moderate_all_eportfolios() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/users/42/eportfolios"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 3
        })))
        .mount(&server)
        .await;

    let result = user
        .moderate_all_eportfolios(&[("spam_status".to_string(), "marked_as_spam".to_string())])
        .await
        .unwrap();
    assert_eq!(result["count"], 3);
}

#[tokio::test]
async fn test_user_get_calendar_events_for_user() {
    let server = MockServer::start().await;
    let user = setup(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42/calendar_events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "title": "Office Hours", "context_code": "course_1"},
            {"id": 2, "title": "Final Exam", "context_code": "course_1"}
        ])))
        .mount(&server)
        .await;

    let events = user
        .get_calendar_events_for_user()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].id, 1);
    assert_eq!(events[0].title.as_deref(), Some("Office Hours"));
}
