use crate::{
    error::{CanvasError, Result},
    http::Requester,
    pagination::PageStream,
    params::wrap_params,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Parameters for creating or updating a Canvas module.
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateModuleParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlock_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_sequential_progress: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_final_grade: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,
}

/// Parameters for creating a new Canvas module.
pub type CreateModuleParams = UpdateModuleParams;

/// Parameters for creating a module item.
#[derive(Debug, Clone, Serialize)]
pub struct CreateModuleItemParams {
    #[serde(rename = "type")]
    pub item_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_tab: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,
}

/// Parameters for updating a module item.
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateModuleItemParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_tab: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,
}

/// A Canvas course module (a collection of ordered content items).
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
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

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Module {
    fn course_id_or_err(&self) -> Result<u64> {
        self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Module does not have a course_id".to_string(),
            errors: vec![],
        })
    }

    fn propagate(&self, m: &mut Module) {
        m.requester = self.requester.clone();
        m.course_id = self.course_id;
    }

    fn propagate_item(&self, item: &mut ModuleItem) {
        item.requester = self.requester.clone();
        item.course_id = self.course_id;
        item.module_id = Some(self.id);
    }

    /// Edit this module's settings.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/modules/:id`
    pub async fn edit(&self, params: UpdateModuleParams) -> Result<Module> {
        let course_id = self.course_id_or_err()?;
        let form = wrap_params("module", &params);
        let mut m: Module = self
            .req()
            .put(&format!("courses/{course_id}/modules/{}", self.id), &form)
            .await?;
        self.propagate(&mut m);
        Ok(m)
    }

    /// Delete this module.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/modules/:id`
    pub async fn delete(&self) -> Result<Module> {
        let course_id = self.course_id_or_err()?;
        let mut m: Module = self
            .req()
            .delete(
                &format!("courses/{course_id}/modules/{}", self.id),
                &[],
            )
            .await?;
        self.propagate(&mut m);
        Ok(m)
    }

    /// Re-lock this module's progressions.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/modules/:id/relock`
    pub async fn relock(&self) -> Result<Module> {
        let course_id = self.course_id_or_err()?;
        let mut m: Module = self
            .req()
            .put(
                &format!("courses/{course_id}/modules/{}/relock", self.id),
                &[],
            )
            .await?;
        self.propagate(&mut m);
        Ok(m)
    }

    /// Stream all items in this module.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/modules/:id/items`
    pub fn get_module_items(&self) -> PageStream<ModuleItem> {
        let course_id = self.course_id.unwrap_or(0);
        let module_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/modules/{module_id}/items"),
            vec![],
            move |mut item: ModuleItem, req| {
                item.requester = Some(Arc::clone(&req));
                item.course_id = Some(course_id);
                item.module_id = Some(module_id);
                item
            },
        )
    }

    /// Fetch a single module item by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/modules/:id/items/:item_id`
    pub async fn get_module_item(&self, item_id: u64) -> Result<ModuleItem> {
        let course_id = self.course_id_or_err()?;
        let mut item: ModuleItem = self
            .req()
            .get(
                &format!(
                    "courses/{course_id}/modules/{}/items/{item_id}",
                    self.id
                ),
                &[],
            )
            .await?;
        self.propagate_item(&mut item);
        Ok(item)
    }

    /// Create a new item in this module.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/modules/:id/items`
    pub async fn create_module_item(&self, params: CreateModuleItemParams) -> Result<ModuleItem> {
        let course_id = self.course_id_or_err()?;
        // content_id is required for all types except SubHeader
        if params.item_type != "SubHeader" && params.content_id.is_none() {
            return Err(CanvasError::BadRequest {
                message: "content_id is required for this module item type".to_string(),
                errors: vec![],
            });
        }
        let form = wrap_params("module_item", &params);
        let mut item: ModuleItem = self
            .req()
            .post(
                &format!("courses/{course_id}/modules/{}/items", self.id),
                &form,
            )
            .await?;
        self.propagate_item(&mut item);
        Ok(item)
    }
}

/// An individual item within a Canvas module.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
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

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
    #[serde(skip)]
    pub course_id: Option<u64>,
}

impl ModuleItem {
    fn course_module_prefix(&self) -> Result<String> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "ModuleItem does not have a course_id".to_string(),
            errors: vec![],
        })?;
        let module_id = self.module_id.ok_or_else(|| CanvasError::BadRequest {
            message: "ModuleItem does not have a module_id".to_string(),
            errors: vec![],
        })?;
        Ok(format!("courses/{course_id}/modules/{module_id}"))
    }

    fn propagate(&self, item: &mut ModuleItem) {
        item.requester = self.requester.clone();
        item.course_id = self.course_id;
        item.module_id = self.module_id;
    }

    /// Edit this module item.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/modules/:module_id/items/:id`
    pub async fn edit(&self, params: UpdateModuleItemParams) -> Result<ModuleItem> {
        let prefix = self.course_module_prefix()?;
        let form = wrap_params("module_item", &params);
        let mut item: ModuleItem = self
            .req()
            .put(&format!("{prefix}/items/{}", self.id), &form)
            .await?;
        self.propagate(&mut item);
        Ok(item)
    }

    /// Delete this module item.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/modules/:module_id/items/:id`
    pub async fn delete(&self) -> Result<ModuleItem> {
        let prefix = self.course_module_prefix()?;
        let mut item: ModuleItem = self
            .req()
            .delete(&format!("{prefix}/items/{}", self.id), &[])
            .await?;
        self.propagate(&mut item);
        Ok(item)
    }

    /// Mark this module item as complete.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/modules/:module_id/items/:id/done`
    pub async fn complete(&self) -> Result<ModuleItem> {
        let prefix = self.course_module_prefix()?;
        let mut item: ModuleItem = self
            .req()
            .put(&format!("{prefix}/items/{}/done", self.id), &[])
            .await?;
        self.propagate(&mut item);
        Ok(item)
    }

    /// Mark this module item as incomplete.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/modules/:module_id/items/:id/done`
    pub async fn uncomplete(&self) -> Result<ModuleItem> {
        let prefix = self.course_module_prefix()?;
        let mut item: ModuleItem = self
            .req()
            .delete(&format!("{prefix}/items/{}/done", self.id), &[])
            .await?;
        self.propagate(&mut item);
        Ok(item)
    }
}

/// Completion requirement for a module item.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompletionRequirement {
    #[serde(rename = "type")]
    pub requirement_type: Option<String>,
    pub min_score: Option<f64>,
    pub completed: Option<bool>,
}
