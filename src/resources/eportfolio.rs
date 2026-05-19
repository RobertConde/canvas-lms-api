use crate::error::Result;
use crate::http::Requester;
use crate::pagination::PageStream;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EPortfolioPage {
    pub id: u64,
    pub eportfolio_id: Option<u64>,
    pub position: Option<u64>,
    pub name: Option<String>,
    pub content: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EPortfolio {
    pub id: u64,
    pub user_id: Option<u64>,
    pub name: Option<String>,
    pub public: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
    pub spam_status: Option<String>,
    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl EPortfolio {
    fn req(&self) -> &Arc<Requester> {
        self.requester.as_ref().expect("requester not injected")
    }

    fn endpoint(&self) -> String {
        format!("eportfolios/{}", self.id)
    }

    /// Delete this ePortfolio.
    ///
    /// `DELETE /api/v1/eportfolios/:id`
    pub async fn delete(&self) -> Result<EPortfolio> {
        let mut p: EPortfolio = self.req().delete(&self.endpoint(), &[]).await?;
        p.requester = self.requester.clone();
        Ok(p)
    }

    /// List pages in this ePortfolio.
    ///
    /// `GET /api/v1/eportfolios/:eportfolio_id/pages`
    pub fn get_pages(&self) -> PageStream<EPortfolioPage> {
        let endpoint = format!("{}/pages", self.endpoint());
        PageStream::new(self.req().clone(), &endpoint, vec![])
    }

    /// Update the spam_status of this ePortfolio (admin only).
    ///
    /// `PUT /api/v1/eportfolios/:eportfolio_id/moderate`
    pub async fn moderate(&self, spam_status: &str) -> Result<EPortfolio> {
        let params = vec![("spam_status".into(), spam_status.to_string())];
        let endpoint = format!("{}/moderate", self.endpoint());
        let mut p: EPortfolio = self.req().put(&endpoint, &params).await?;
        p.requester = self.requester.clone();
        Ok(p)
    }

    /// Restore a deleted ePortfolio (admin only).
    ///
    /// `PUT /api/v1/eportfolios/:eportfolio_id/restore`
    pub async fn restore(&self) -> Result<EPortfolio> {
        let endpoint = format!("{}/restore", self.endpoint());
        let mut p: EPortfolio = self.req().put(&endpoint, &[]).await?;
        p.requester = self.requester.clone();
        Ok(p)
    }
}
