use crate::{error::Result, http::Requester, pagination::PageStream};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A custom gradebook column in a Canvas course.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct CustomGradebookColumn {
    pub id: u64,
    pub course_id: Option<u64>,
    pub title: Option<String>,
    pub position: Option<u64>,
    pub teacher_notes: Option<bool>,
    pub read_only: Option<bool>,
    pub hidden: Option<bool>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

/// A single data entry (cell value) in a custom gradebook column.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct ColumnData {
    pub id: Option<u64>,
    pub gradebook_column_id: Option<u64>,
    pub content: Option<String>,
    pub user_id: Option<u64>,

    /// Injected from parent; not returned by Canvas.
    #[serde(skip)]
    pub(crate) course_id: Option<u64>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

/// Parameters for creating or updating a custom gradebook column.
#[derive(Debug, Default, Clone, Serialize)]
pub struct CustomGradebookColumnParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teacher_notes: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
}

impl CustomGradebookColumn {
    fn course_id(&self) -> u64 {
        self.course_id
            .expect("CustomGradebookColumn missing course_id")
    }

    /// Delete this custom gradebook column.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/custom_gradebook_columns/:id`
    pub async fn delete(&self) -> Result<CustomGradebookColumn> {
        let mut col: CustomGradebookColumn = self
            .req()
            .delete(
                &format!(
                    "courses/{}/custom_gradebook_columns/{}",
                    self.course_id(),
                    self.id
                ),
                &[],
            )
            .await?;
        col.requester = self.requester.clone();
        Ok(col)
    }

    /// Update this custom gradebook column.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/custom_gradebook_columns/:id`
    pub async fn update(
        &self,
        params: CustomGradebookColumnParams,
    ) -> Result<CustomGradebookColumn> {
        use crate::params::wrap_params;
        let form = wrap_params("column", &params);
        let mut col: CustomGradebookColumn = self
            .req()
            .put(
                &format!(
                    "courses/{}/custom_gradebook_columns/{}",
                    self.course_id(),
                    self.id
                ),
                &form,
            )
            .await?;
        col.requester = self.requester.clone();
        Ok(col)
    }

    /// Stream all data entries for this custom gradebook column.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/custom_gradebook_columns/:id/data`
    pub fn get_column_entries(&self) -> PageStream<ColumnData> {
        let course_id = self.course_id();
        let column_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/custom_gradebook_columns/{column_id}/data"),
            vec![],
            move |mut d: ColumnData, req| {
                d.requester = Some(Arc::clone(&req));
                d.course_id = Some(course_id);
                d.gradebook_column_id = Some(column_id);
                d
            },
        )
    }
}

impl ColumnData {
    fn course_id(&self) -> u64 {
        self.course_id.expect("ColumnData missing course_id")
    }

    fn column_id(&self) -> u64 {
        self.gradebook_column_id
            .expect("ColumnData missing gradebook_column_id")
    }

    fn user_id(&self) -> u64 {
        self.user_id.expect("ColumnData missing user_id")
    }

    /// Update the content of this column data entry.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/custom_gradebook_columns/:column_id/data/:user_id`
    pub async fn update_column_data(&self, content: &str) -> Result<ColumnData> {
        let params = vec![("column_data[content]".to_string(), content.to_string())];
        let mut data: ColumnData = self
            .req()
            .put(
                &format!(
                    "courses/{}/custom_gradebook_columns/{}/data/{}",
                    self.course_id(),
                    self.column_id(),
                    self.user_id()
                ),
                &params,
            )
            .await?;
        data.requester = self.requester.clone();
        data.course_id = self.course_id;
        data.gradebook_column_id = self.gradebook_column_id;
        Ok(data)
    }
}
