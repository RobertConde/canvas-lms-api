use crate::error::Result;
use crate::http::Requester;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ── Planner Note ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlannerNote {
    pub id: u64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub user_id: Option<u64>,
    pub workflow_state: Option<String>,
    pub course_id: Option<u64>,
    pub todo_date: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PlannerNoteParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub todo_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub course_id: Option<u64>,
}

impl PlannerNote {
    fn req(&self) -> &Arc<Requester> {
        self.requester.as_ref().expect("requester not injected")
    }

    fn endpoint(&self) -> String {
        format!("planner_notes/{}", self.id)
    }

    /// Delete this planner note.
    ///
    /// `DELETE /api/v1/planner_notes/:id`
    pub async fn delete(&self) -> Result<PlannerNote> {
        let mut n: PlannerNote = self.req().delete(&self.endpoint(), &[]).await?;
        n.requester = self.requester.clone();
        Ok(n)
    }

    /// Update this planner note.
    ///
    /// `PUT /api/v1/planner_notes/:id`
    pub async fn update(&self, params: PlannerNoteParams) -> Result<PlannerNote> {
        let flat: Vec<(String, String)> = serde_json::to_value(&params)
            .unwrap_or_default()
            .as_object()
            .into_iter()
            .flatten()
            .filter_map(|(k, v)| {
                v.as_str()
                    .map(|s| (k.clone(), s.to_string()))
                    .or_else(|| v.as_u64().map(|n| (k.clone(), n.to_string())))
            })
            .collect();
        let mut n: PlannerNote = self.req().put(&self.endpoint(), &flat).await?;
        n.requester = self.requester.clone();
        Ok(n)
    }
}

// ── Planner Override ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlannerOverride {
    pub id: u64,
    pub plannable_type: Option<String>,
    pub plannable_id: Option<u64>,
    pub user_id: Option<u64>,
    pub assignment_id: Option<u64>,
    pub workflow_state: Option<String>,
    pub marked_complete: Option<bool>,
    pub dismissed: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PlannerOverrideParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marked_complete: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dismissed: Option<bool>,
}

impl PlannerOverride {
    fn req(&self) -> &Arc<Requester> {
        self.requester.as_ref().expect("requester not injected")
    }

    fn endpoint(&self) -> String {
        format!("planner/overrides/{}", self.id)
    }

    /// Delete this planner override.
    ///
    /// `DELETE /api/v1/planner/overrides/:id`
    pub async fn delete(&self) -> Result<PlannerOverride> {
        let mut o: PlannerOverride = self.req().delete(&self.endpoint(), &[]).await?;
        o.requester = self.requester.clone();
        Ok(o)
    }

    /// Update this planner override.
    ///
    /// `PUT /api/v1/planner/overrides/:id`
    pub async fn update(&self, params: PlannerOverrideParams) -> Result<PlannerOverride> {
        let mut flat: Vec<(String, String)> = vec![];
        if let Some(mc) = params.marked_complete {
            flat.push(("marked_complete".into(), mc.to_string()));
        }
        if let Some(d) = params.dismissed {
            flat.push(("dismissed".into(), d.to_string()));
        }
        let mut o: PlannerOverride = self.req().put(&self.endpoint(), &flat).await?;
        o.requester = self.requester.clone();
        Ok(o)
    }
}
