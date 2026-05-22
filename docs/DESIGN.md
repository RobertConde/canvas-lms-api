# canvas-lms-api — Design Document

## Context

The Python [`canvasapi`](https://github.com/ucfopen/canvasapi) package is a widely-used wrapper
for Instructure's Canvas LMS REST API v1. This crate is a Rust equivalent, published on crates.io
as **`canvas-lms-api`**, following Rust/Cargo best practices: typed structs, async-first, proper
error enums, streaming pagination, and full test coverage.

---

## What the Python Package Does (Key Facts)

- **Entry point**: `Canvas(base_url, access_token)` — single client object
- **Base URLs**: `/api/v1/` (main), `/api/quiz/v1/` (New Quizzes), `/api/graphql`
- **Auth**: Bearer token in every request header
- **75 resource types**: Course, User, Assignment, Submission, Group, Quiz, Enrollment, Section, etc.
- **Client has 63+ methods**: `get_course(id)`, `create_course(...)`, `get_user(id)`, etc.
- **Resources have sub-methods**: `course.create_assignment(...)`, `course.get_assignments()`, etc.
- **Pagination**: Lazy `PaginatedList` driven by `Link` response headers (RFC 5988) + `meta.pagination.next` JSON fallback
- **HTTP**: Synchronous `requests.Session`; no async
- **Parameter serialization**: Nested dicts/lists → bracket notation (`course[name]=Foo`, `ids[]=1&ids[]=2`)
- **Error hierarchy**: `CanvasException` → `BadRequest` (400), `InvalidAccessToken`/`Unauthorized` (401), `Forbidden` (403), `ResourceDoesNotExist` (404), `Conflict` (409), `UnprocessableEntity` (422), `RateLimitExceeded` (429)
- **Deserialization**: Dynamic attribute injection from JSON; ISO8601 strings auto-get `_date` companion attrs
- **Tests**: 73 test files, all mock-based (`requests_mock`), 57 JSON fixture files in `tests/fixtures/`

---

## Repository Structure

```
canvas-lms-api/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── DESIGN.md                      ← this file
├── CHANGELOG.md
├── LICENSE                        # AGPLv3
├── .github/
│   └── workflows/
│       ├── ci.yml                 # fmt, clippy, test, doc, MSRV
│       └── publish.yml            # crates.io on tag push
├── src/
│   ├── lib.rs
│   ├── client.rs                  # Canvas struct + top-level methods
│   ├── client_blocking.rs         # CanvasBlocking (feature="blocking")
│   ├── error.rs                   # CanvasError enum + Result alias
│   ├── http.rs                    # Requester — pub(crate) only
│   ├── pagination.rs              # PageStream<T>, Link header parsing
│   ├── params.rs                  # Bracket-notation serialization
│   ├── upload.rs                  # Two-step Canvas file upload
│   ├── graphql.rs                 # GraphQL support (feature="graphql")
│   └── resources/
│       ├── mod.rs
│       ├── types.rs               # Shared enums: WorkflowState, SubmissionType, etc.
│       ├── account.rs
│       ├── assignment.rs
│       ├── course.rs
│       ├── discussion_topic.rs
│       ├── enrollment.rs
│       ├── file.rs
│       ├── folder.rs
│       ├── group.rs
│       ├── module.rs
│       ├── page.rs
│       ├── progress.rs
│       ├── quiz.rs
│       ├── section.rs
│       ├── submission.rs
│       ├── tab.rs
│       ├── user.rs
│       └── params/                # Typed builder structs per resource
│           ├── mod.rs
│           ├── course_params.rs
│           └── assignment_params.rs
├── tests/
│   ├── integration/
│   │   ├── course_test.rs
│   │   └── ...
│   └── fixtures/                  # JSON fixtures adapted from Python repo's tests/fixtures/
└── examples/
    ├── list_courses.rs
    └── create_assignment.rs
```

---

## Cargo.toml Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `async` | yes | Async API (tokio + reqwest + futures + async-stream) |
| `blocking` | no | `CanvasBlocking` wrapper; drives tokio internally |
| `new-quizzes` | no | `/api/quiz/v1/` endpoint support |
| `graphql` | no | `/api/graphql` endpoint support |
| `full` | no | `new-quizzes` + `graphql` |

---

## Core Types

### `CanvasError` (`src/error.rs`)

Maps HTTP status codes to typed variants, mirroring the Python exception hierarchy:

| HTTP Status | Variant |
|-------------|---------|
| 400 | `BadRequest { message, errors }` |
| 401 + `WWW-Authenticate` header | `InvalidAccessToken(msg)` |
| 401 (no header) | `Unauthorized(msg)` |
| 403 | `Forbidden(msg)` |
| 404 | `ResourceDoesNotExist` |
| 409 | `Conflict(msg)` |
| 422 | `UnprocessableEntity(msg)` |
| 429 | `RateLimitExceeded { remaining }` |
| other | `ApiError { status, message }` |
| transport | `Http(reqwest::Error)` |
| JSON parse | `Json(serde_json::Error)` |

### `PageStream<T>` (`src/pagination.rs`)

Async-lazy page fetcher implementing `futures::Stream` directly (`async` feature). Callers use it with `futures::StreamExt` or the built-in `collect_all()`:

```rust
use futures::StreamExt;
let courses: Vec<_> = canvas.get_courses().collect_all().await?;
// or via StreamExt — next(), map(), filter(), collect(), etc.
let mut stream = canvas.get_courses();
while let Some(result) = stream.next().await {
    let course = result?;
}
```

- Default `per_page=100` (matching Python default)
- Parses `Link` header (RFC 5988) for `rel="next"` URL
- Falls back to `meta.pagination.next` in the response body
- Buffers a single page at a time in a `VecDeque<T>`

### `params.rs` — Bracket-notation serialization

Ports Python's `combine_kwargs`/`flatten_kwarg` from `canvasapi/util.py`:

```
{"course": {"name": "Foo", "ids": [1, 2]}}
→ [("course[name]", "Foo"), ("course[ids][]", "1"), ("course[ids][]", "2")]
```

Callers serialize typed `#[derive(Serialize, Default)]` param structs to `serde_json::Value`,
then pass through `flatten_params()`.

### Resource structs (`src/resources/`)

All resources are plain `#[derive(Deserialize, Serialize, Debug, Clone)]` structs with all
fields `Option<T>`. They carry a `#[serde(skip)] requester: Option<Arc<Requester>>` field
injected after deserialization. This enables the same ergonomics as the Python library:

```rust
let course = canvas.get_course(1).await?;
let assignments = course.get_assignments(); // PageStream<Assignment>
```

All enums (e.g. `WorkflowState`) use `#[serde(other)] Unknown` as a catch-all to avoid
panics from unexpected Canvas API values.

### `Canvas` client (`src/client.rs`)

Single entry point:

```rust
let canvas = Canvas::new("https://canvas.example.edu", "token")?;
let canvas = Canvas::with_client(base_url, token, custom_reqwest_client)?;
```

`base_url` validation: reject URLs containing `/api/v1`, reject non-HTTPS, strip trailing slashes.

### `CanvasBlocking` (`src/client_blocking.rs`, feature=`blocking`)

Wraps `Canvas` with a `tokio::runtime::Runtime` to drive async methods synchronously.
No `reqwest::blocking` dependency needed — one runtime per `CanvasBlocking` instance.

---

## Testing Strategy

### Unit tests (in-crate, `#[cfg(test)]`)
- `params.rs`: bracket-notation output verified against known Canvas query strings (ported from Python's `test_util.py`)
- `error.rs`: every HTTP status code → `CanvasError` variant
- `pagination.rs`: Link header parsing edge cases (no next, malformed, meta.pagination fallback)

### Integration tests (`tests/integration/`)
Each resource gets a test file using `wiremock`:

```rust
#[tokio::test]
async fn test_get_course() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1, "name": "Test Course", "workflow_state": "available"
        })))
        .mount(&server).await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    assert_eq!(course.id, 1);
}
```

JSON fixtures in `tests/fixtures/` are adapted from `canvasapi/tests/fixtures/` (57 files).

### Real integration tests (optional)
Gated by `CANVAS_BASE_URL` + `CANVAS_ACCESS_TOKEN` environment variables. Excluded from CI
unless secrets are provided.

---

## CI/CD

### `ci.yml` — runs on every push + PR
1. `cargo fmt --check`
2. `cargo clippy --features full -- -D warnings`
3. `cargo test --features full`
4. `cargo test --no-default-features --features blocking`
5. `cargo doc --no-deps --features full` (with `RUSTDOCFLAGS="-D warnings"`)
6. MSRV check: `cargo check` on Rust 1.75

### `publish.yml` — runs on `v*` tag push
`cargo publish` with `CARGO_REGISTRY_TOKEN` secret.

---

## Docs Strategy

Every public struct, method, and enum has `///` doc comments. Methods include a
`# Canvas API` section citing the exact endpoint:

```rust
/// List all assignments for this course.
///
/// # Canvas API
/// `GET /api/v1/courses/:course_id/assignments`
///
/// # Example
/// ```no_run
/// # use canvas_lms_api::Canvas;
/// # #[tokio::main] async fn main() -> canvas_lms_api::Result<()> {
/// let canvas = Canvas::new("https://canvas.example.edu", "token")?;
/// let course = canvas.get_course(1).await?;
/// let assignments = course.get_assignments().collect_all().await?;
/// # Ok(()) }
/// ```
pub fn get_assignments(&self) -> PageStream<Assignment> { ... }
```

---

## Phased Scope

### v0.1.0 — Core MVP
Course, User, Assignment, Submission, Enrollment, Section, Module, Quiz, Group,
Account, File, Folder, Page, DiscussionTopic, Progress, Tab

### v0.2.0
Account calendars, Blueprints, Content migrations, Outcome groups, Rubrics,
Gradebook history, SIS imports, New Quizzes (`new-quizzes` feature),
External tools, Communication channels

### v0.3.0 (shipped)
AppointmentGroup, CalendarEvent, Conversation, EnrollmentTerm, EPortfolio/EPortfolioPage,
GradingPeriod, GradingStandard, JWT, ContentExport, GradeChangeLog, Feature/FeatureFlag,
PlannerNote/PlannerOverride, Role. 214 tests, 0 clippy warnings.

### v0.4.0 (shipped)
- **Polls** — `Poll`, `PollChoice`, `PollSession`, `PollSubmission` with full CRUD matching Python `canvasapi` surface (`update`, `delete`, `get_choice/choices`, `create_choice`, `get_session/sessions`, `create_session`, `open`/`close`, `get_submission`, `create_submission`). Client-level: `get_poll`, `get_polls`, `create_poll`.
- **Collaborations** — `Collaboration` + `Collaborator` structs. `Course::get_collaborations()` and `Group::get_collaborations()` (Canvas API exposes list only; no create/get-single endpoint exists). `Collaboration::get_collaborators()` (`GET /collaborations/:id/members`).
- **LTI Resource Links** — `LtiResourceLink` + `CreateLtiResourceLinkParams`. `Course::get_lti_resource_links()`, `Course::get_lti_resource_link()`, `Course::create_lti_resource_link()`.
- **`impl futures::Stream for PageStream<T>`** — direct trait impl (gated on `async` feature) so callers use `StreamExt` (`next()`, `map()`, `filter()`, `collect()`, etc.) without an adapter. `Group` promoted from data-only to requester-bearing to support methods.
- **`#[derive(CanvasResource)]`** — `canvas-lms-api-derive` proc-macro crate generates `fn req()` on any struct with a `requester: Option<Arc<Requester>>` field; applied to all 18 existing resource structs + new v0.4.0 structs.
- 242 tests, 0 clippy warnings.

### v0.5.0 — API Depth (shipped) ✓

v0.5.0 fills the method gaps across all existing resources. v0.1–v0.4 added structs and basic CRUD; the resources below were identified as having zero or near-zero instance methods despite the Python library having substantial coverage.

Every method added in v0.5.0 has a matching wiremock integration test (mirroring the Python `canvasapi` test suite). Resources needing parent context (Tab → `course_id`, Page → `course_id`/`group_id`, DiscussionTopic → `course_id`/`group_id`, Module → `course_id`, ModuleItem → `course_id`+`module_id`, Submission → `course_id`) get `#[serde(skip)]` fields for those ids, injected by callers via `PageStream::new_with_injector` or direct field assignment.

#### Implementation order

**Batch 1 — Quick wins (small gaps, 1-4 methods each)**

| Resource | Methods | API |
|---|---|---|
| `Tab` | `update()` | `PUT courses/:c/tabs/:id` |
| `Enrollment` | `accept()`, `reject()`, `deactivate()`, `reactivate()` | `POST/DELETE/PUT courses/:c/enrollments/:id/...` |
| `Progress` | `query()` | `GET progress/:id` |
| `FeatureFlag` | `delete()`, `set_feature_flag(state)` | `DELETE/PUT :ctx_type/:ctx_id/features/flags/:feature` |

`Tab` also needs `#[serde(skip)] pub(crate) course_id: Option<u64>` and `Course::get_tabs()` updated to inject it. `Enrollment` already has `course_id`. `Course::get_enrollments()`, `Canvas::get_enrollment()`, `Canvas::get_progress()` updated to inject requester.

Tests: `tests/tab_test.rs`, `tests/enrollment_test.rs`, `tests/progress_test.rs`, `tests/feature_flag_test.rs`

**Batch 2 — File resources**

| Resource | Methods | API |
|---|---|---|
| `File` | `update()`, `delete()`, `get_contents()`, `download(path)` | `PUT/DELETE files/:id`, raw URL GET |
| `Folder` | `update()`, `delete()`, `get_files()`, `get_folders()`, `create_folder()`, `copy_file()` | `PUT/DELETE folders/:id`, `GET/POST folders/:id/...` |

`get_contents()` / `download()` use a new `Requester::get_url_bytes(url)` helper in `src/http.rs` that GETs an absolute URL with auth. `Course::get_files()`, `Course::get_file()`, `Course::get_folder()`, `Course::get_folders()` updated to inject requester.

Tests: `tests/file_test.rs`, `tests/folder_test.rs`

**Batch 3 — Page + PageRevision**

`Page` gets `#[serde(skip)] pub(crate) course_id: Option<u64>` and `pub(crate) group_id: Option<u64>` + `fn parent_prefix()` helper. New `PageRevision` struct in same file.

| Resource | Methods | API |
|---|---|---|
| `Page` | `edit()`, `delete()`, `get_revisions()`, `get_revision_by_id()`, `show_latest_revision()`, `revert_to_revision()` | `PUT/DELETE/GET :parent/pages/:url/...` |
| `PageRevision` | data struct only | — |

`Course::get_pages()`, `Course::get_page()` updated to inject requester + `course_id`. Same for `Group`.

Tests: `tests/page_test.rs`

**Batch 4 — Section**

`Section` already has `course_id`. Add requester + CanvasResource derive.

Methods: `edit()`, `delete()`, `enroll_user()`, `get_enrollments()`, `cross_list_section()`, `decross_list_section()`, `get_assignment_override()`, `get_multiple_submissions()`

`Course::get_sections()`, `Course::get_section()` updated to inject requester.

Tests: `tests/section_test.rs`

**Batch 5 — Module + ModuleItem**

`Module` gets `#[serde(skip)] pub(crate) course_id: Option<u64>`. `ModuleItem` gets `course_id` + `module_id`.

| Resource | Methods |
|---|---|
| `Module` | `edit()`, `delete()`, `relock()`, `get_module_items()`, `get_module_item()`, `create_module_item()` |
| `ModuleItem` | `edit()`, `delete()`, `complete()`, `uncomplete()` |

Add `Course::create_module()`. Update `Course::get_modules()`, `Course::get_module()` to inject requester + `course_id`.

Tests: `tests/module_test.rs`

**Batch 6 — DiscussionTopic + DiscussionEntry**

`DiscussionTopic` gets `#[serde(skip)] pub(crate) group_id: Option<u64>` (already has `course_id`) + `fn parent_prefix()`. New `DiscussionEntry` struct in same file with `requester`, `course_id`, `group_id`, `topic_id`.

| Resource | Methods |
|---|---|
| `DiscussionTopic` | `update()`, `delete()`, `post_entry()`, `get_topic_entries()`, `get_entries()`, `mark_as_read()`, `mark_as_unread()`, `mark_entries_as_read()`, `mark_entries_as_unread()`, `subscribe()`, `unsubscribe()` |
| `DiscussionEntry` | `update()`, `delete()`, `post_reply()`, `get_replies()`, `mark_as_read()`, `mark_as_unread()`, `rate()` |

Add `Course::create_discussion_topic()`. Update `Course::get_discussion_topics()`, `Course::get_discussion_topic()` to inject requester + `course_id`. Same for `Group`.

Tests: `tests/discussion_topic_test.rs`

**Batch 7 — Submission**

`Submission` gets `#[serde(skip)] pub(crate) course_id: Option<u64>` (already has `assignment_id`, `user_id`).

Methods: `edit()`, `mark_read()`, `mark_unread()`, `create_submission_peer_review()`, `delete_submission_peer_review()`, `get_submission_peer_reviews()`

Tests: `tests/submission_test.rs`

**Batch 8 — Assignment depth**

`Assignment` gets `#[serde(skip)] pub(crate) course_id: Option<u64>`. New structs `AssignmentGroup` and `AssignmentOverride` in same file.

| Resource | Methods |
|---|---|
| `Assignment` | `edit()`, `delete()`, `get_submissions()`, `get_submission()`, `submit()`, `get_overrides()`, `get_override()`, `create_override()`, `get_peer_reviews()`, `get_gradeable_students()`, `set_extensions()`, `submissions_bulk_update()` |
| `AssignmentGroup` | `edit()`, `delete()` |
| `AssignmentOverride` | `edit()`, `delete()` |

Add `Course::get_assignment_groups()`, `Course::create_assignment_group()`. Update `Course::get_assignments()`, `Course::get_assignment()`, `Course::create_assignment()` to inject requester + `course_id`.

Tests: extend `tests/assignment_test.rs`

**Batch 9 — Quiz depth**

`Quiz` gets `#[serde(skip)] pub(crate) course_id: Option<u64>`. New structs `QuizQuestion`, `QuizSubmission`.

| Resource | Methods |
|---|---|
| `Quiz` | `edit()`, `delete()`, `create_question()`, `get_question()`, `get_questions()`, `create_submission()`, `get_submission()`, `get_submissions()`, `get_statistics()`, `set_extensions()` |
| `QuizSubmission` | `complete()`, `get_submission_questions()`, `get_times()`, `update_score_and_comments()` |
| `QuizQuestion` | data struct only |

Update `Course::get_quizzes()`, `Course::get_quiz()`, `Course::create_quiz()` to inject requester + `course_id`.

Tests: extend `tests/quiz_test.rs`

**Batch 10 — User depth**

`User` gets requester + CanvasResource derive. ~30 methods covering: `edit()`, `get_profile()`, `terminate_sessions()`, `merge_into()`, `get_avatars()`, `get_page_views()`, `get_observees()`, `add_observee()`, `remove_observee()`, `show_observee()`, `get_observers()`, `create_pairing_code()`, `get_colors()`, `get_color()`, `update_color()`, `get_missing_submissions()`, `get_enrollments()`, `get_courses()`, `get_files()`, `get_folders()`, `create_folder()`, `get_file_quota()`, `get_communication_channels()`, `get_user_logins()`, `get_authentication_events()`, `get_features()`, `export_content()`, `get_content_exports()`, `get_eportfolios()`, `get_open_poll_sessions()`, `get_closed_poll_sessions()`.

`Canvas::get_user()` updated to inject requester. Tests: extend `tests/user_test.rs`

**Batch 11 — Group depth + GroupMembership + GroupCategory**

`Group` already has requester + CanvasResource derive. Add ~25 methods: `edit()`, `delete()`, `get_users()`, `get_memberships()`, `create_membership()`, `get_membership()`, `update_membership()`, `remove_user()`, `invite()`, `get_files()`, `get_file()`, `get_folders()`, `get_folder()`, `create_folder()`, `get_pages()`, `get_page()`, `create_page()`, `get_discussion_topics()`, `get_discussion_topic()`, `create_discussion_topic()`, `get_tabs()`, `get_content_migrations()`, `get_content_exports()`, `preview_html()`, `resolve_path()`.

New struct `GroupMembership` (methods: `update()`, `remove_self()`, `remove_user()`).
New struct `GroupCategory` (methods: `get_groups()`, `get_users()`, `create_group()`, `delete()`, `assign_members()`, `update()`).

Add `Course::get_group_categories()`, `Course::create_group_category()`.

Tests: extend `tests/group_test.rs`

**Batch 12 — Course remaining depth**

Add to `Course`: `conclude()`, `reset()`, `get_settings()`, `update_settings()`, `get_late_policy()`, `create_late_policy()`, `edit_late_policy()`, `get_custom_columns()`, `create_custom_column()`, `get_multiple_submissions()`, `submissions_bulk_update()`, `enroll_user()`.

Tests: extend `tests/course_test.rs`

**Batch 13 — Account remaining depth**

Add to `Account`: `update()`, `create_subaccount()`, `get_subaccounts()`, `get_users()`, `create_user()`, `delete_user()`, `get_courses()`, `get_groups()`, `get_group_categories()`, `create_group_category()`, `get_enrollment_terms()`, `create_enrollment_term()`, `get_admins()`, `create_admin()`, `get_authentication_providers()`, `get_reports()`, `create_report()`.

Tests: extend `tests/account_test.rs`

**Small gaps in already-covered resources** (addressed inline with the batch above, or standalone):

| Resource | Methods added |
|---|---|
| `ExternalTool` | `get_sessionless_launch_url()` |
| `ContentMigration` | `get_progress()`, `get_selective_data()` |
| `OutcomeGroup` | `import_outcome_group()` |
| `CommunicationChannel` | `update_multiple_preferences()` |

### v0.6.0 (shipped) ✓

**504 tests, 0 failures.** All three CI matrix configs clean.

| Batch | What was delivered |
|---|---|
| 1 | `CustomGradebookColumn` struct + methods; dedicated test files for `ContentMigration`, `ExternalTool`, `Outcome`/`OutcomeGroup`, `SisImport`, `CustomGradebookColumn` |
| 2 | Quiz extended depth: `QuizGroup`, `QuizReport`, quiz reports/events, flag/unflag, `answer_submission_questions` |
| 3 | Two-step Canvas file upload (`src/upload.rs`); `upload_file` on `Folder`, `User`, `Group` |
| 4 | User remaining: `get_file`, `get_folder`, `resolve_path`, grade change events, content migration methods, `get_feature_flag` |
| 5 | Group remaining: `show_front_page`, `edit_front_page`, `get_file_quota`, `get_external_feeds`, `create/delete_external_feed`, `get_assignment_override`, `set/remove_usage_rights`, `get_licenses` |
| 6 | Assignment extended: `get_grade_change_events`, moderated grading endpoints (provisional grades, `select_students_for_moderation`, `publish_provisional_grades`, `show_provisional_grades_for_student`) |

---

### v0.7.0 (shipped) ✓

**566 tests, 0 failures.** All three CI matrix configs clean.

| Batch | What was delivered |
|---|---|
| 1 | Account missing methods: `create_course`, `create_sis_import`, `delete_admin`, `delete_grading_period`, `get_enrollment`, `get_authentication_events`; + 6 new account tests |
| 2 | Course missing methods: `show_front_page`, `edit_front_page`, `export_content`, `get_full_discussion_topic`, `preview_html`, `reorder_pinned_topics`, `get_user`, `get_recent_students`, `upload_file`, `set_usage_rights`, `remove_usage_rights`, `get_licenses`, `get_external_feeds`, `create/delete_external_feed`, `create_course_section`; + 15 tests |
| 3 | Rubric instance methods: `Rubric::delete/update`; `RubricAssociation::update/delete/create_rubric_assessment`; `RubricAssessment::update/delete`; new `tests/rubric_test.rs` (7 tests) |
| 4 | Group: `get/create_content_migration`, `get_migration_systems`, `get/export_content`, `get_full_discussion_topic`, `get_activity_stream_summary`, `reorder_pinned_topics` (8 tests); User: `add_observee_with_credentials`, `get_calendar_events`, `get_content_export`, `get_licenses`, `set/remove_usage_rights` (6 tests) |
| 5 | Login resource: new `src/resources/login.rs` with `edit`, `delete`, `get_authentication_events`; `Account::get_user_logins`, `create_user_login`; `tests/login_test.rs` (5 tests) |
| 6 | `ExternalTool::get_sessionless_launch_url` (2 tests); `OutcomeImport` struct + `get_progress`; `Account::import_outcomes`; `Course::import_outcomes`, `get_outcome_import_status` (4 tests) |

### v0.8.0 — Plan

Goals: fill Account test gaps + missing methods; add missing Course methods; implement Rubric resource; fill Group/User remaining depth; add Login resource; tighten ExternalTool and OutcomeImport.

#### Batch 1 — Account: test coverage + missing methods

Python ref: `tests/test_account.py`

**Add tests** for already-implemented Account methods that currently have no test coverage:
- Content exports: `get_content_export`, `get_content_exports`, `create_content_export`
- Content migrations: `get_content_migration`, `get_content_migrations`, `create_content_migration`, `get_migrators`
- Enrollment terms: `get_enrollment_term`, `get_enrollment_terms`, `create_enrollment_term`
- External tools: `get_external_tool`, `get_external_tools`, `create_external_tool`
- Feature flags: `get_feature_flag`, `get_features`
- Grading standards: `get_grading_standard`, `get_grading_standards`
- Outcome groups: `get_outcome_group`, `get_outcome_group_links`
- Roles: `get_role`, `get_roles`, `update_role`
- SIS imports: `get_sis_import`, `get_sis_imports`, `get_sis_imports_running`

**Add missing Account methods** (not yet in Rust):
- `create_course(params)` → `POST /api/v1/accounts/:id/courses` → `Course`
- `create_sis_import(params)` → `POST /api/v1/accounts/:id/sis_imports` → `SisImport`
- `delete_admin(user_id)` → `DELETE /api/v1/accounts/:id/admins/:user_id` → `Value`
- `delete_grading_period(id)` → `DELETE /api/v1/accounts/:id/grading_periods/:id` → `()`
- `get_enrollment(id)` → `GET /api/v1/accounts/:id/enrollments/:id` → `Enrollment`
- `get_authentication_events()` → `GET /api/v1/audit/authentication/accounts/:id` → `PageStream<Value>`

#### Batch 2 — Course: missing methods

Python ref: `tests/test_course.py`

Add to `src/resources/course.rs` (all need tests in `tests/course_extra_test.rs` or new `course_methods2_test.rs`):

- **Front page**: `show_front_page()` → `GET .../front_page` → `Page`; `edit_front_page(params)` → `PUT`
- **Content**: `export_content(type)`, `get_full_discussion_topic(id)`, `preview_html(html)`, `reorder_pinned_topics(order)`
- **Users**: `get_user(id)` → `GET .../users/:id`; `get_recent_students()`
- **Files/rights**: `upload_file(request, data)` (two-step, same pattern as Group); `set_usage_rights(params)`; `remove_usage_rights(params)`; `get_licenses()`
- **External feeds**: `get_external_feeds()`, `create_external_feed(url)`, `delete_external_feed(id)`
- **Sections**: `create_course_section(name)` → `POST .../sections` → `Section`

#### Batch 3 — Rubric resource (entirely new)

Python ref: `tests/test_rubric.py`

New `Rubric` and `RubricAssociation` structs in `src/resources/rubric.rs`. Fields: `id`, `title`, `context_id`, `context_type`, `points_possible`, `reusable`, `read_only`, `free_form_criterion_comments`, `hide_score_total`, `data` (criteria array).

`RubricAssociation` methods: `update(params)`, `delete()`.

Add to `Account`: `get_rubric(id)`, `get_rubrics()`.
Add to `Course`: `get_rubric(id)`, `get_rubrics()`, `create_rubric(params)`, `create_rubric_association(params)`, `get_rubric_association(id)`, `get_rubric_associations()`.

Note: `create_rubric` response envelope is `{"rubric": {...}, "rubric_association": {...}}` — extract `rubric` key.

Tests: `tests/rubric_test.rs` (target: ~12 tests)

#### Batch 4 — Group + User remaining gaps

Python refs: `tests/test_group.py`, `tests/test_user.py`

**Group** (add to `src/resources/group.rs`):
- `create_content_migration(migration_type)`, `get_content_migration(id)`, `get_migration_systems()`
- `export_content(type)`, `get_content_export(id)`
- `get_full_discussion_topic(id)`, `get_activity_stream_summary()`, `reorder_pinned_topics(order)`

**User** (add to `src/resources/user.rs`):
- `add_observee_with_credentials(params)` — same endpoint as `add_observee` but sends credentials
- `get_calendar_events(params)` → `GET /api/v1/users/:id/calendar_events`
- `get_content_export(id)`, `get_licenses()`
- `set_usage_rights(params)`, `remove_usage_rights(params)`

#### Batch 5 — Login resource (entirely new)

Python ref: `tests/test_login.py`

New `Login` struct in `src/resources/login.rs`. Fields: `id`, `account_id`, `user_id`, `workflow_state`, `unique_id`, `sis_user_id`, `integration_id`, `authentication_provider_id`.

`Login` methods:
- `edit(params)` → `PUT /api/v1/accounts/:account_id/logins/:id` → `Login`
- `delete()` → `DELETE /api/v1/accounts/:account_id/logins/:id` → `()`

Add to `Account`:
- `get_user_logins(user_id)` → `GET /api/v1/accounts/:id/logins?user_id=:user_id` → `PageStream<Login>`
- `create_user_login(params)` → `POST /api/v1/accounts/:id/logins` → `Login`

`Login` needs `account_id` injected at call sites so its instance methods can form the URL.

Tests: `tests/login_test.rs` (target: ~6 tests)

#### Batch 6 — ExternalTool sessionless launch + OutcomeImport

Python refs: `tests/test_external_tool.py`, `tests/test_outcome_import.py`

**ExternalTool**:
- `get_sessionless_launch_url(params)` → `GET /api/v1/:context/external_tools/sessionless_launch`
  Uses existing `context_type` + `context_id` fields. Returns `Value`.

**OutcomeImport** (formalize the existing raw-JSON approach):
- New `OutcomeImport` struct with `id`, `account_id`, `course_id`, `workflow_state`, `progress`.
- `OutcomeImport::get_progress()` → `GET /api/v1/:context/outcome_imports/:id`
- Add `Account::import_outcomes(params)` → `POST /api/v1/accounts/:id/outcome_imports` → `OutcomeImport`
- Add `Course::import_outcomes(params)` + `Course::get_outcome_import_status(id)` → `OutcomeImport`

Tests: extend `tests/external_tool_test.rs`; extend `tests/outcome_test.rs` or new `tests/outcome_import_test.rs`

---

### v1.0.0
Full API surface. Semver stability guarantee. MSRV policy pinned to N-2 stable.

---

## Implementation Sequence

1. `error.rs` + `params.rs` + unit tests — no HTTP, proves core logic
2. `http.rs` + `pagination.rs` — reqwest wired up, `PageStream` with wiremock tests
3. `client.rs` + `resources/course.rs` + `resources/user.rs` — first end-to-end flows
4. `resources/assignment.rs` + `submission.rs` + `enrollment.rs`
5. Remaining Phase 1 resources: section, module, quiz, group, account, file, folder, page, discussion_topic, tab
6. `upload.rs` — two-step Canvas file upload
7. `client_blocking.rs` (feature=`blocking`)
8. `graphql.rs` (feature=`graphql`)
9. Docs pass — every public item has doc comment + example
10. CI/CD, README, publish to crates.io

---

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| License | AGPLv3 | Copyleft; matches open-source Canvas ecosystem norms |
| Async vs sync primary | Async (tokio + reqwest) default | Matches modern Rust ecosystem; sync via `blocking` feature |
| Resource method location | On resource struct (carrying `Arc<Requester>`) | Same ergonomics as Python (`course.get_assignments()`); `Arc` clone is free |
| Parameter API | Typed `#[derive(Serialize, Default)]` structs | IDE autocomplete, compile-time checks, rustdoc-generated param docs |
| Pagination type | `PageStream<T>` with `collect_all()` + manual streaming | Simple to use without requiring `StreamExt` import for common cases |
| DateTime handling | `chrono::DateTime<Utc>` via serde default ISO 8601 | Canvas sends RFC 3339 strings; chrono handles `Z` suffix natively |
| Enum fallback | `#[serde(other)] Unknown` variant on all state enums | Prevents deserialization panics from new Canvas states |
| Test HTTP mocking | `wiremock` + JSON fixtures | Mirrors Python's `requests_mock` approach; fixtures reused from Python repo |
| File upload | `src/upload.rs` two-step (POST metadata, POST multipart) | Canvas requires this for all file uploads |

---

## Critical Python Source Files to Reference

| File | Rust equivalent |
|------|----------------|
| `canvasapi/requester.py` | `src/http.rs` |
| `canvasapi/paginated_list.py` | `src/pagination.rs` |
| `canvasapi/util.py` | `src/params.rs` |
| `canvasapi/canvas_object.py` | `src/resources/*.rs` (struct + serde) |
| `canvasapi/exceptions.py` | `src/error.rs` |
| `canvasapi/course.py` | `src/resources/course.rs` |
| `tests/fixtures/` | `tests/fixtures/` |
