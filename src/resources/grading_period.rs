use crate::error::Result;
use crate::http::Requester;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GradingPeriod {
    pub id: u64,
    pub title: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub close_date: Option<String>,
    pub weight: Option<f64>,
    pub is_last: Option<bool>,
    /// Injected from the parent Course.
    pub course_id: Option<u64>,
    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GradingPeriodParams {
    pub start_date: String,
    pub end_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

impl GradingPeriod {
    fn req(&self) -> &Arc<Requester> {
        self.requester.as_ref().expect("requester not injected")
    }

    fn endpoint(&self) -> String {
        let course_id = self.course_id.unwrap_or(0);
        format!("courses/{}/grading_periods/{}", course_id, self.id)
    }

    /// Delete this grading period.
    ///
    /// `DELETE /api/v1/courses/:course_id/grading_periods/:id`
    pub async fn delete(&self) -> Result<()> {
        // Canvas returns 204 No Content on success — parse as unit via serde_json::Value
        let _: serde_json::Value = self.req().delete(&self.endpoint(), &[]).await?;
        Ok(())
    }

    /// Update this grading period.
    ///
    /// `PUT /api/v1/courses/:course_id/grading_periods/:id`
    pub async fn update(&self, params: GradingPeriodParams) -> Result<GradingPeriod> {
        let entry = serde_json::to_value(&params).unwrap_or_default();
        let body = serde_json::json!({ "grading_periods": [entry] });
        let flat = crate::params::to_canvas_params("grading_periods", &body["grading_periods"]);
        let raw: serde_json::Value = self.req().put(&self.endpoint(), &flat).await?;
        let gp_val = raw["grading_periods"]
            .as_array()
            .and_then(|a| a.first())
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let mut gp: GradingPeriod = serde_json::from_value(gp_val)?;
        gp.requester = self.requester.clone();
        gp.course_id = self.course_id;
        Ok(gp)
    }
}
