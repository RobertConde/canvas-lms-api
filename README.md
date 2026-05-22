# canvas-lms-api

A Rust client for the [Instructure Canvas LMS REST API](https://canvas.instructure.com/doc/api/).

[![Crates.io](https://img.shields.io/crates/v/canvas-lms-api)](https://crates.io/crates/canvas-lms-api)
[![docs.rs](https://img.shields.io/docsrs/canvas-lms-api)](https://docs.rs/canvas-lms-api)
[![CI](https://github.com/RobertConde/canvas-lms-api/actions/workflows/ci.yml/badge.svg)](https://github.com/RobertConde/canvas-lms-api/actions/workflows/ci.yml)

## Quickstart

```toml
[dependencies]
canvas-lms-api = "0.5"
tokio = { version = "1", features = ["full"] }
```

```rust
use canvas_lms_api::Canvas;

#[tokio::main]
async fn main() -> canvas_lms_api::Result<()> {
    let canvas = Canvas::new("https://canvas.example.edu", "your-access-token")?;

    // Fetch a single course
    let course = canvas.get_course(123).await?;
    println!("Course: {}", course.name.unwrap_or_default());

    // Collect all assignments for the course
    let assignments = course.get_assignments().collect_all().await?;
    for a in assignments {
        println!("  Assignment: {}", a.name.unwrap_or_default());
    }

    Ok(())
}
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `async` | yes | Async API via `tokio` + `reqwest` |
| `blocking` | no | Synchronous wrapper (`CanvasBlocking`) |
| `new-quizzes` | no | New Quizzes endpoint (`/api/quiz/v1/`) |
| `graphql` | no | GraphQL endpoint support |
| `full` | no | All optional features |

## Resources covered (v0.5)

**Core:** Course, User, Assignment (+ Group + Override), Submission, Enrollment,
Section, Module (+ ModuleItem), Quiz (+ Question + Submission), Group
(+ Membership + Category), Account, File, Folder, Page (+ Revision),
DiscussionTopic (+ Entry), Progress, Tab

**Extended:** AccountCalendar, AppointmentGroup, Blueprint, CalendarEvent,
Collaboration, CommunicationChannel, ContentExport, ContentMigration, Conversation,
EnrollmentTerm, EPortfolio, ExternalTool, Feature / FeatureFlag,
GradeChangeLog, GradebookHistory, GradingPeriod, GradingStandard,
JWT, LtiResourceLink, Outcome / OutcomeGroup, Planner (Note + Override),
Poll / PollChoice / PollSession / PollSubmission, Role, Rubric, SisImport

**Feature-gated:** NewQuiz (`new-quizzes`), GraphQL queries (`graphql`)

## Examples

Runnable example programs live in [`examples/`](examples/). Each is a standalone Cargo project.

| Example | Description |
|---------|-------------|
| [`list-courses-and-assignments`](examples/list-courses-and-assignments) | List all your courses and their assignments |

More examples to come.

## Access tokens

Generate a token in Canvas: **Account → Settings → Approved Integrations → New Access Token**.

## License

Licensed under the [MIT License](LICENSE).

## Acknowledgements

Inspired by [canvasapi](https://github.com/ucfopen/canvasapi), the Python Canvas API library
maintained by the University of Central Florida.
