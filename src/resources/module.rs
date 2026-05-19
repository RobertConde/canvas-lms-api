use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A Canvas course module (a collection of ordered content items).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Module {
    pub id: u64,
    pub course_id: Option<u64>,
    pub name: Option<String>,
    pub position: Option<u64>,
    pub unlock_at: Option<DateTime<Utc>>,
    pub require_sequential_progress: Option<bool>,
    pub prerequisite_module_ids: Option<Vec<u64>>,
    pub items_count: Option<u64>,
    pub items_url: Option<String>,
    pub state: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub publish_final_grade: Option<bool>,
    pub published: Option<bool>,
}

/// An individual item within a Canvas module.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModuleItem {
    pub id: u64,
    pub module_id: Option<u64>,
    pub position: Option<u64>,
    pub title: Option<String>,
    pub indent: Option<u64>,
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    pub content_id: Option<u64>,
    pub html_url: Option<String>,
    pub url: Option<String>,
    pub page_url: Option<String>,
    pub external_url: Option<String>,
    pub completion_requirement: Option<CompletionRequirement>,
    pub published: Option<bool>,
}

/// Completion requirement for a module item.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompletionRequirement {
    #[serde(rename = "type")]
    pub requirement_type: Option<String>,
    pub min_score: Option<f64>,
    pub completed: Option<bool>,
}
