// GraphQL endpoint support. Enabled with features = ["graphql"].

use crate::{error::Result, http::Requester};
use serde_json::Value;
use std::sync::Arc;

/// GraphQL client bound to a Canvas instance.
///
/// Obtain one via [`Canvas::graphql()`][crate::client::Canvas::graphql].
pub struct GraphQL {
    pub(crate) requester: Arc<Requester>,
}

impl GraphQL {
    /// Execute a raw GraphQL query against the Canvas `/api/graphql` endpoint.
    ///
    /// Returns the full JSON response (including `data` and optional `errors` fields).
    ///
    /// # Example
    /// ```no_run
    /// # #[tokio::main] async fn main() -> canvas_lms_api::Result<()> {
    /// let canvas = canvas_lms_api::Canvas::new("https://canvas.example.edu", "token")?;
    /// let gql = canvas.graphql();
    /// let result = gql.query("{ allCourses { id name } }", None).await?;
    /// println!("{result}");
    /// # Ok(()) }
    /// ```
    pub async fn query(&self, query: &str, variables: Option<Value>) -> Result<Value> {
        self.requester.graphql_query(query, variables).await
    }
}
