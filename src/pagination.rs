use crate::{error::Result, http::Requester};
use serde::de::DeserializeOwned;
use std::{collections::VecDeque, marker::PhantomData, sync::Arc};
use url::Url;

/// An async stream of Canvas API resources, fetched lazily page by page.
///
/// Collect all items with [`PageStream::collect_all`], or stream them one at a time.
///
/// ```no_run
/// # async fn example(stream: canvas_lms_api::PageStream<()>) -> canvas_lms_api::Result<()> {
/// let items: Vec<()> = stream.collect_all().await?;
/// # Ok(()) }
/// ```
pub struct PageStream<T> {
    requester: Arc<Requester>,
    next_url: Option<Url>,
    params: Vec<(String, String)>,
    buffer: VecDeque<T>,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> PageStream<T> {
    pub(crate) fn new(
        requester: Arc<Requester>,
        endpoint: &str,
        mut params: Vec<(String, String)>,
    ) -> Self {
        if !params.iter().any(|(k, _)| k == "per_page") {
            params.push(("per_page".into(), "100".into()));
        }
        let next_url = requester.base_url.join(endpoint).ok();
        Self {
            requester,
            next_url,
            params,
            buffer: VecDeque::new(),
            _phantom: PhantomData,
        }
    }

    /// Fetch the next page and push items into the buffer. Returns false when exhausted.
    async fn fetch_next(&mut self) -> Result<bool> {
        let url = match self.next_url.take() {
            Some(u) => u,
            None => return Ok(false),
        };

        let resp = self.requester.get_raw(url, &self.params).await?;

        // Parse Link header for next page URL (RFC 5988).
        self.next_url = parse_link_next(resp.headers());

        let body: serde_json::Value = resp.json().await?;

        // Support meta.pagination.next fallback.
        if self.next_url.is_none() {
            if let Some(next) = body
                .get("meta")
                .and_then(|m| m.get("pagination"))
                .and_then(|p| p.get("next"))
                .and_then(|n| n.as_str())
            {
                self.next_url = Url::parse(next).ok();
            }
        }

        let items = match body {
            serde_json::Value::Array(arr) => arr,
            serde_json::Value::Object(ref obj) => {
                // Some endpoints wrap the array in an object key.
                obj.values()
                    .find_map(|v| v.as_array().cloned())
                    .unwrap_or_default()
            }
            _ => vec![],
        };

        for item in items {
            let resource: T = serde_json::from_value(item)?;
            self.buffer.push_back(resource);
        }

        Ok(true)
    }

    /// Collect all items across all pages into a Vec.
    pub async fn collect_all(mut self) -> Result<Vec<T>> {
        let mut out = Vec::new();
        loop {
            while let Some(item) = self.buffer.pop_front() {
                out.push(item);
            }
            if !self.fetch_next().await? {
                break;
            }
        }
        while let Some(item) = self.buffer.pop_front() {
            out.push(item);
        }
        Ok(out)
    }
}

fn parse_link_next(headers: &reqwest::header::HeaderMap) -> Option<Url> {
    let link = headers.get("Link")?.to_str().ok()?;
    for part in link.split(',') {
        let mut url_part = None;
        let mut rel = None;
        for segment in part.split(';') {
            let s = segment.trim();
            if s.starts_with('<') && s.ends_with('>') {
                url_part = Some(&s[1..s.len() - 1]);
            } else if s.starts_with("rel=") {
                rel = Some(s.trim_start_matches("rel=").trim_matches('"'));
            }
        }
        if rel == Some("next") {
            if let Some(u) = url_part {
                return Url::parse(u).ok();
            }
        }
    }
    None
}
