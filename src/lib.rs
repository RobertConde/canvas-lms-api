pub mod client;
pub mod error;
pub mod pagination;
pub mod resources;

mod http;
pub(crate) mod params;
mod upload;

#[cfg(feature = "graphql")]
pub mod graphql;

#[cfg(feature = "blocking")]
pub mod client_blocking;

pub use client::Canvas;
pub use error::{CanvasError, Result};
pub use pagination::PageStream;
