# canvas-lms-api

A Rust client for the [Instructure Canvas LMS REST API](https://canvas.instructure.com/doc/api/).

[![Crates.io](https://img.shields.io/crates/v/canvas-lms-api)](https://crates.io/crates/canvas-lms-api)
[![docs.rs](https://img.shields.io/docsrs/canvas-lms-api)](https://docs.rs/canvas-lms-api)
[![CI](https://github.com/RobertConde/canvas-lms-api/actions/workflows/ci.yml/badge.svg)](https://github.com/RobertConde/canvas-lms-api/actions/workflows/ci.yml)

## Quickstart

```toml
[dependencies]
canvas-lms-api = "0.1"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

```rust
use canvas_lms_api::Canvas;
use futures::StreamExt;

#[tokio::main]
async fn main() -> canvas_lms_api::Result<()> {
    let canvas = Canvas::new("https://canvas.example.edu", "your-access-token")?;

    // Fetch a single course
    let course = canvas.get_course(123).await?;
    println!("Course: {}", course.name.unwrap_or_default());

    // Stream all assignments (lazy, page by page)
    let mut assignments = course.get_assignments();
    while let Some(result) = assignments.next().await {
        let a = result?;
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

## Resources covered (v0.1)

Course, User, Assignment, Submission, Enrollment, Section, Module, Quiz,
Group, Account, File, Folder, Page, DiscussionTopic, Progress, Tab

## Access tokens

Generate a token in Canvas: **Account → Settings → Approved Integrations → New Access Token**.

## License

Licensed under the [MIT License](LICENSE).

## Acknowledgements

Inspired by [canvasapi](https://github.com/ucfopen/canvasapi), the Python Canvas API library
maintained by the University of Central Florida.
