use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GradingSchemeEntry {
    pub name: Option<String>,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GradingStandard {
    pub id: u64,
    pub title: Option<String>,
    pub context_type: Option<String>,
    pub context_id: Option<u64>,
    pub grading_scheme: Option<Vec<GradingSchemeEntry>>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GradingStandardParams {
    pub title: String,
    pub grading_scheme_entry: Vec<GradingSchemeEntry>,
}
