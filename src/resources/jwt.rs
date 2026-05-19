use serde::{Deserialize, Serialize};

/// A Canvas JWT token, used with other Canvas services.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CanvasJwt {
    pub token: Option<String>,
    pub expires_at: Option<String>,
}
