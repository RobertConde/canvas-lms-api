use serde::{Deserialize, Serialize};

/// A content license available in Canvas.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct License {
    pub id: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
}
