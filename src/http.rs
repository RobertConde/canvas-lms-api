use crate::error::{CanvasError, CanvasErrorBody, Result};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use tracing::{debug, info};
use url::Url;

#[derive(Debug)]
pub(crate) struct Requester {
    pub(crate) client: Client,
    pub(crate) base_url: Url,
    pub(crate) new_quizzes_url: Url,
    pub(crate) graphql_url: Url,
    access_token: String,
}

impl Requester {
    pub(crate) fn new(base_url: Url, access_token: String, client: Client) -> Self {
        let new_quizzes_url = base_url
            .join("../quiz/v1/")
            .unwrap_or_else(|_| base_url.clone());
        let graphql_url = base_url
            .join("../graphql")
            .unwrap_or_else(|_| base_url.clone());
        Self {
            client,
            base_url,
            new_quizzes_url,
            graphql_url,
            access_token,
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.access_token)
    }

    pub(crate) async fn get_raw(&self, url: Url, params: &[(String, String)]) -> Result<Response> {
        info!("GET {url}");
        debug!("params: {params:?}");
        let resp = self
            .client
            .get(url)
            .header("Authorization", self.auth_header())
            .query(params)
            .send()
            .await?;
        check_status(resp).await
    }

    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(String, String)],
    ) -> Result<T> {
        let url = self.base_url.join(endpoint)?;
        let resp = self.get_raw(url, params).await?;
        Ok(resp.json().await?)
    }

    pub(crate) async fn post<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(String, String)],
    ) -> Result<T> {
        let url = self.base_url.join(endpoint)?;
        info!("POST {url}");
        let resp = self
            .client
            .post(url)
            .header("Authorization", self.auth_header())
            .form(params)
            .send()
            .await?;
        let resp = check_status(resp).await?;
        Ok(resp.json().await?)
    }

    pub(crate) async fn put<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(String, String)],
    ) -> Result<T> {
        let url = self.base_url.join(endpoint)?;
        info!("PUT {url}");
        let resp = self
            .client
            .put(url)
            .header("Authorization", self.auth_header())
            .form(params)
            .send()
            .await?;
        let resp = check_status(resp).await?;
        Ok(resp.json().await?)
    }

    pub(crate) async fn delete<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(String, String)],
    ) -> Result<T> {
        let url = self.base_url.join(endpoint)?;
        info!("DELETE {url}");
        let resp = self
            .client
            .delete(url)
            .header("Authorization", self.auth_header())
            .query(params)
            .send()
            .await?;
        let resp = check_status(resp).await?;
        Ok(resp.json().await?)
    }

    #[allow(dead_code)] // used by future resource update methods
    pub(crate) async fn patch<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(String, String)],
    ) -> Result<T> {
        let url = self.base_url.join(endpoint)?;
        info!("PATCH {url}");
        let resp = self
            .client
            .patch(url)
            .header("Authorization", self.auth_header())
            .form(params)
            .send()
            .await?;
        let resp = check_status(resp).await?;
        Ok(resp.json().await?)
    }

    // New Quizzes API (`/api/quiz/v1/`) — enabled by the `new-quizzes` feature.

    #[cfg(feature = "new-quizzes")]
    pub(crate) async fn nq_get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(String, String)],
    ) -> Result<T> {
        let url = self.new_quizzes_url.join(endpoint)?;
        let resp = self.get_raw(url, params).await?;
        Ok(resp.json().await?)
    }

    #[cfg(feature = "new-quizzes")]
    pub(crate) async fn nq_post<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &serde_json::Value,
    ) -> Result<T> {
        let url = self.new_quizzes_url.join(endpoint)?;
        info!("POST (NQ) {url}");
        let resp = self
            .client
            .post(url)
            .header("Authorization", self.auth_header())
            .json(body)
            .send()
            .await?;
        let resp = check_status(resp).await?;
        Ok(resp.json().await?)
    }

    #[cfg(feature = "new-quizzes")]
    pub(crate) async fn nq_patch<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &serde_json::Value,
    ) -> Result<T> {
        let url = self.new_quizzes_url.join(endpoint)?;
        info!("PATCH (NQ) {url}");
        let resp = self
            .client
            .patch(url)
            .header("Authorization", self.auth_header())
            .json(body)
            .send()
            .await?;
        let resp = check_status(resp).await?;
        Ok(resp.json().await?)
    }

    #[cfg(feature = "new-quizzes")]
    pub(crate) async fn nq_delete<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = self.new_quizzes_url.join(endpoint)?;
        info!("DELETE (NQ) {url}");
        let resp = self
            .client
            .delete(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        let resp = check_status(resp).await?;
        Ok(resp.json().await?)
    }

    // GraphQL API — enabled by the `graphql` feature.

    #[cfg(feature = "graphql")]
    pub(crate) async fn graphql_query(
        &self,
        query: &str,
        variables: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "query": query,
            "variables": variables.unwrap_or(serde_json::Value::Null)
        });
        info!("POST (GraphQL) {}", self.graphql_url);
        let resp = self
            .client
            .post(self.graphql_url.clone())
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?;
        let resp = check_status(resp).await?;
        Ok(resp.json().await?)
    }
}

async fn check_status(resp: Response) -> Result<Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }

    let www_auth = resp
        .headers()
        .get("WWW-Authenticate")
        .map(|v| v.to_str().unwrap_or("").to_string());
    let rate_remaining = resp
        .headers()
        .get("X-Rate-Limit-Remaining")
        .map(|v| v.to_str().unwrap_or("").to_string());

    let body = resp.text().await.unwrap_or_default();
    let parsed: Option<CanvasErrorBody> = serde_json::from_str(&body).ok();

    let message = parsed
        .as_ref()
        .and_then(|b| {
            b.errors
                .as_ref()
                .and_then(|e| e.first())
                .and_then(|e| e.message.clone())
                .or_else(|| b.error.clone())
        })
        .unwrap_or_else(|| body.clone());

    let errors = parsed.and_then(|b| b.errors).unwrap_or_default();

    Err(match status.as_u16() {
        400 => CanvasError::BadRequest { message, errors },
        401 if www_auth.is_some() => CanvasError::InvalidAccessToken(message),
        401 => CanvasError::Unauthorized(message),
        403 => CanvasError::Forbidden(message),
        404 => CanvasError::ResourceDoesNotExist,
        409 => CanvasError::Conflict(message),
        422 => CanvasError::UnprocessableEntity(message),
        429 => CanvasError::RateLimitExceeded {
            remaining: rate_remaining,
        },
        s => CanvasError::ApiError { status: s, message },
    })
}
