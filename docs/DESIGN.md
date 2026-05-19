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

Async-lazy page fetcher implementing a manual async stream pattern. Callers use it with `futures::StreamExt`:

```rust
use futures::StreamExt;
let courses: Vec<_> = canvas.get_courses().collect_all().await?;
// or one at a time:
let mut stream = canvas.get_courses();
while let Some(result) = stream.next_item().await {
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

### v0.4.0
- Polls (`Poll`, `PollChoice`, `PollSession`, `PollSubmission`) with full CRUD
- Collaborations: list/get/create on courses and groups
- LTI resource links: course-scoped list/create
- `impl futures::Stream for PageStream<T>` — lazy page-by-page iteration via `StreamExt`
  (`next()`, `map()`, `filter()`, etc.) without collecting everything upfront
- Struct-level mutation tests for second-order objects (implemented but untested):
  Rubric, RubricAssociation, RubricAssessment, OutcomeGroup, ContentMigration,
  MigrationIssue, SisImport, Blueprint types
- `#[derive(CanvasResource)]` proc-macro to reduce `Arc<Requester>` injection boilerplate

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
