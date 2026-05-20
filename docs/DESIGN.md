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

### v0.5.0 — API Depth

v0.5.0 fills the method gaps across all existing resources. v0.1–v0.4 added structs and basic CRUD; the resources below were identified as having zero or near-zero instance methods despite the Python library having substantial coverage.

**Struct-only resources (0 instance methods today — full method impl needed):**

| Resource | Key methods to add |
|---|---|
| `Submission` | `edit()` (grade/comment), `mark_read/unread()`, `upload_comment()`, peer review CRUD |
| `Group` | `edit()`, `delete()`, member management, full content surface (files, pages, discussions, tabs, migrations) — `get_collaborations()` already added in v0.4.0 |
| `GroupMembership` *(new struct)* | `update()`, `remove_self()`, `remove_user()` |
| `GroupCategory` *(new struct)* | `get_groups()`, `create_group()`, `get_users()`, `assign_members()`, `update()`, `delete()` |
| `Section` | `edit()`, `delete()`, `enroll_user()`, `get_enrollments()`, `cross_list_section()`, `decross_list_section()`, `get_assignment_override()` |
| `Module` | `edit()`, `delete()`, `get_module_items()`, `get_module_item()`, `create_module_item()`, `relock()` |
| `ModuleItem` | `edit()`, `delete()`, `complete()`, `uncomplete()` |
| `DiscussionTopic` | `update()`, `delete()`, `post_entry()`, `get_topic_entries()`, `get_entries()`, `mark_as_read/unread()`, `mark_entries_as_read/unread()`, `subscribe/unsubscribe()`, `get_parent()` |
| `DiscussionEntry` *(new struct)* | `update()`, `delete()`, `post_reply()`, `get_replies()`, `mark_as_read/unread()`, `rate()`, `get_discussion()` |
| `Page` | `edit()`, `delete()`, `get_revisions()`, `get_revision_by_id()`, `show_latest_revision()`, `revert_to_revision()`, `get_parent()` |
| `PageRevision` *(new struct)* | `get_parent()` |
| `File` | `update()`, `delete()`, `get_contents()`, `download()` |
| `Folder` | `update()`, `delete()`, `get_files()`, `get_folders()`, `create_folder()`, `copy_file()`, `upload()` |
| `Enrollment` | `accept()`, `reject()`, `deactivate()`, `reactivate()` |
| `Tab` | `update()` |

**Assignment depth** — `Assignment` struct has no instance methods today:
- `edit()`, `delete()`, `create_override()`, `get_override()`, `get_overrides()`, `get_submissions()`, `get_submission()`, `submit()`, `upload_to_submission()`, `get_peer_reviews()`, `get_gradeable_students()`, `set_extensions()`, `submissions_bulk_update()`, moderated grading methods, `get_grade_change_events()`
- New structs: `AssignmentGroup` (`edit`, `delete`), `AssignmentOverride` (`edit`, `delete`)
- New Course methods: `get_assignment_groups()`, `create_assignment_group()`, `create_assignment_overrides()`, `update_assignment_overrides()`, `get_assignments_for_group()`

**Quiz depth** — `Quiz` struct has no instance methods today:
- `edit()`, `delete()`, question CRUD, question group CRUD, submission lifecycle, reports, `get_statistics()`, `set_extensions()`, `broadcast_message()`, moderated quiz methods
- New structs: `QuizQuestion`, `QuizSubmission`, `QuizSubmissionQuestion`

**User depth** — `User` has ~5 methods vs ~48 in Python:
- Profile: `edit()`, `merge_into()`, `get_profile()`, `update_settings()`, `terminate_sessions()`
- Observees/observers: `get_observees()`, `add_observee()`, `remove_observee()`, `show_observee()`, `get_observers()`, `create_pairing_code()`
- Dashboard colors: `get_colors()`, `get_color()`, `update_color()`
- Content: `get_files()`, `get_folders()`, `create_folder()`, `upload()`, `get_file_quota()`, `resolve_path()`
- Cross-course: `get_assignments()`, `get_missing_submissions()`
- Misc: `get_avatars()`, `get_page_views()`, `get_user_logins()`, `get_authentication_events()`, grade change events, features, content exports, ePortfolios

**Course depth** — ~80 methods still missing:
- Lifecycle: `conclude()`, `reset()`
- Settings: `get_settings()`, `update_settings()`
- Late policy: `get_late_policy()`, `create_late_policy()`, `edit_late_policy()`
- Custom gradebook columns: `get_custom_columns()`, `create_custom_column()`, `column_data_bulk_update()`
- Bulk submissions: `get_multiple_submissions()`, `submissions_bulk_update()`
- Analytics: course-level and user-in-course assignment/participation/summary data
- Outcomes: `get_all_outcome_links_in_context()`, `get_outcome_results()`, `get_outcome_result_rollups()`, `import_outcome()`
- Group categories, external feeds, epub exports, usage rights, file quota, grading standards CRUD, New Quizzes CRUD

**Account depth** — ~50 methods still missing:
- Sub-accounts, admin management, user management, auth providers, SSO settings
- Department-level analytics (6 methods), notifications, reports, outcome imports
- `create_account()`, `update()`, `delete()`

**Small gaps in already-covered resources:**

| Resource | Missing |
|---|---|
| `ExternalTool` | `get_parent()`, `get_sessionless_launch_url()` |
| `ContentMigration` | `get_parent()`, `get_progress()`, `get_selective_data()` |
| `BlueprintTemplate` | `change_blueprint_restrictions()` |
| `OutcomeGroup` | `import_outcome_group()` |
| `OutcomeLink` | `get_outcome()`, `get_outcome_group()` |
| `CommunicationChannel` | `update_multiple_preferences()` |
| `FeatureFlag` | `delete()`, `set_feature_flag()` |
| `Progress` | `query()` — poll for updated status |

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
