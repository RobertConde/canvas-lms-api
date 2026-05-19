use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanvasError {
    #[error("Bad request: {message}")]
    BadRequest {
        message: String,
        errors: Vec<ApiError>,
    },

    #[error("Invalid access token: {0}")]
    InvalidAccessToken(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Resource does not exist")]
    ResourceDoesNotExist,

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    #[error("Rate limit exceeded (X-Rate-Limit-Remaining: {remaining:?})")]
    RateLimitExceeded { remaining: Option<String> },

    #[error("Canvas API error (status {status}): {message}")]
    ApiError { status: u16, message: String },

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiError {
    pub message: Option<String>,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct CanvasErrorBody {
    pub errors: Option<Vec<ApiError>>,
    pub error: Option<String>,
    #[allow(dead_code)] // surfaced in future error reporting
    pub error_report_id: Option<u64>,
}

pub type Result<T> = std::result::Result<T, CanvasError>;
