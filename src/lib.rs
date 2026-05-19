//! # canvas-lms-api
//!
//! An async Rust client for the [Instructure Canvas LMS REST API](https://canvas.instructure.com/doc/api/).
//!
//! Licensed under the [MIT License](https://github.com/RobertConde/canvas-lms-api/blob/main/LICENSE).
//!
//! ## Quickstart
//!
//! ```no_run
//! # #[tokio::main] async fn main() -> canvas_lms_api::Result<()> {
//! use canvas_lms_api::Canvas;
//!
//! let canvas = Canvas::new("https://canvas.example.edu", "your-access-token")?;
//!
//! // Fetch a course
//! let course = canvas.get_course(1).await?;
//! println!("{}", course.name.as_deref().unwrap_or_default());
//!
//! // Stream all assignments lazily
//! let assignments = course.get_assignments().collect_all().await?;
//! println!("{} assignments", assignments.len());
//! # Ok(()) }
//! ```
//!
//! ## Features
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `async` | yes | Async API via `tokio` + `reqwest` |
//! | `blocking` | no | `CanvasBlocking` — sync wrapper, no `async`/`.await` needed |
//! | `new-quizzes` | no | New Quizzes endpoint (`/api/quiz/v1/`) |
//! | `graphql` | no | GraphQL endpoint support |
//! | `full` | no | All optional features |
//!
//! ## Pagination
//!
//! Methods returning multiple resources return a [`PageStream<T>`], which fetches pages
//! lazily from Canvas's paginated API. Call [`.collect_all()`][PageStream::collect_all]
//! to gather everything into a `Vec`, or iterate page-by-page as needed.

pub mod client;
pub mod error;
pub mod pagination;
pub mod resources;

mod http;
pub(crate) mod params;
pub mod upload;

#[cfg(feature = "graphql")]
pub mod graphql;

#[cfg(feature = "blocking")]
pub mod client_blocking;

pub use client::Canvas;
pub use error::{CanvasError, Result};
pub use pagination::PageStream;
pub use upload::UploadRequest;

#[cfg(feature = "blocking")]
pub use client_blocking::CanvasBlocking;
