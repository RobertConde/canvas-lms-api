use crate::{error::Result, http::Requester};
use serde::de::DeserializeOwned;
use std::{
    collections::VecDeque,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use url::Url;

type InjectFn<T> = Arc<dyn Fn(T, Arc<Requester>) -> T + Send + Sync>;
type PendingFetch<T> = Pin<Box<dyn Future<Output = Result<(VecDeque<T>, Option<Url>)>> + Send>>;

/// An async stream of Canvas API resources, fetched lazily page by page.
///
/// Collect all items with [`PageStream::collect_all`], or use
/// [`futures::StreamExt`] methods (`.next()`, `.map()`, `.filter()`, etc.)
/// directly — `PageStream` implements [`futures::Stream`] when the `async`
/// feature is enabled (the default).
///
/// ```no_run
/// # async fn example(stream: canvas_lms_api::PageStream<()>) -> canvas_lms_api::Result<()> {
/// // Eager collection into a Vec
/// let items: Vec<()> = stream.collect_all().await?;
/// # Ok(()) }
/// ```
pub struct PageStream<T> {
    requester: Arc<Requester>,
    next_url: Option<Url>,
    params: Vec<(String, String)>,
    buffer: VecDeque<T>,
    inject_fn: Option<InjectFn<T>>,
    _phantom: PhantomData<T>,
    pending: Option<PendingFetch<T>>,
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
            inject_fn: None,
            _phantom: PhantomData,
            pending: None,
        }
    }

    pub(crate) fn new_with_injector(
        requester: Arc<Requester>,
        endpoint: &str,
        mut params: Vec<(String, String)>,
        inject: impl Fn(T, Arc<Requester>) -> T + Send + Sync + 'static,
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
            inject_fn: Some(Arc::new(inject)),
            _phantom: PhantomData,
            pending: None,
        }
    }

    #[cfg(feature = "new-quizzes")]
    pub(crate) fn new_with_injector_nq(
        requester: Arc<Requester>,
        endpoint: &str,
        mut params: Vec<(String, String)>,
        inject: impl Fn(T, Arc<Requester>) -> T + Send + Sync + 'static,
    ) -> Self {
        if !params.iter().any(|(k, _)| k == "per_page") {
            params.push(("per_page".into(), "100".into()));
        }
        let next_url = requester.new_quizzes_url.join(endpoint).ok();
        Self {
            requester,
            next_url,
            params,
            buffer: VecDeque::new(),
            inject_fn: Some(Arc::new(inject)),
            _phantom: PhantomData,
            pending: None,
        }
    }

    /// Collect all items across all pages into a Vec.
    pub async fn collect_all(mut self) -> Result<Vec<T>> {
        let mut out = Vec::new();
        loop {
            while let Some(item) = self.buffer.pop_front() {
                out.push(item);
            }
            match self.next_url.take() {
                None => break,
                Some(url) => {
                    let (items, next_url) = fetch_page(
                        Arc::clone(&self.requester),
                        url,
                        self.params.clone(),
                        self.inject_fn.clone(),
                    )
                    .await?;
                    self.buffer = items;
                    self.next_url = next_url;
                }
            }
        }
        while let Some(item) = self.buffer.pop_front() {
            out.push(item);
        }
        Ok(out)
    }
}

/// Fetch one page from `url`, apply the injector, return buffered items + next URL.
async fn fetch_page<T: DeserializeOwned>(
    requester: Arc<Requester>,
    url: Url,
    params: Vec<(String, String)>,
    inject_fn: Option<InjectFn<T>>,
) -> Result<(VecDeque<T>, Option<Url>)> {
    let resp = requester.get_raw(url, &params).await?;

    let next_url = parse_link_next(resp.headers());

    let body: serde_json::Value = resp.json().await?;

    // Support meta.pagination.next fallback.
    let next_url = if next_url.is_none() {
        body.get("meta")
            .and_then(|m| m.get("pagination"))
            .and_then(|p| p.get("next"))
            .and_then(|n| n.as_str())
            .and_then(|s| Url::parse(s).ok())
    } else {
        next_url
    };

    let items = match body {
        serde_json::Value::Array(arr) => arr,
        serde_json::Value::Object(ref obj) => obj
            .values()
            .find_map(|v| v.as_array().cloned())
            .unwrap_or_default(),
        _ => vec![],
    };

    let mut buffer = VecDeque::new();
    for item in items {
        let resource: T = serde_json::from_value(item)?;
        let resource = if let Some(f) = &inject_fn {
            f(resource, Arc::clone(&requester))
        } else {
            resource
        };
        buffer.push_back(resource);
    }

    Ok((buffer, next_url))
}

// The future in `pending` lives on the heap (Box), so moving PageStream<T>
// is always safe regardless of T.
impl<T> Unpin for PageStream<T> {}

#[cfg(feature = "async")]
impl<T: DeserializeOwned + Send + 'static> futures::Stream for PageStream<T> {
    type Item = Result<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let me = self.get_mut();
        loop {
            // Drain buffer before fetching more.
            if let Some(item) = me.buffer.pop_front() {
                return Poll::Ready(Some(Ok(item)));
            }
            // Poll in-flight page fetch.
            if let Some(fut) = me.pending.as_mut() {
                match fut.as_mut().poll(cx) {
                    Poll::Ready(Ok((items, next_url))) => {
                        me.pending = None;
                        me.buffer = items;
                        me.next_url = next_url;
                        continue;
                    }
                    Poll::Ready(Err(e)) => {
                        me.pending = None;
                        return Poll::Ready(Some(Err(e)));
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }
            // Kick off a new page fetch if more pages remain.
            if let Some(url) = me.next_url.take() {
                let req = Arc::clone(&me.requester);
                let params = me.params.clone();
                let inject = me.inject_fn.clone();
                me.pending = Some(Box::pin(fetch_page(req, url, params, inject)));
                continue;
            }
            return Poll::Ready(None);
        }
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
