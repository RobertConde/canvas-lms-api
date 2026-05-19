// Two-step Canvas file upload.
// Step 1: POST metadata to obtain upload_url + upload_params.
// Step 2: POST multipart form to upload_url (no auth header).

use crate::{error::Result, resources::file::File};
use reqwest::{
    multipart::{Form, Part},
    Client,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize)]
pub struct UploadRequest {
    pub name: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_folder_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UploadIntent {
    upload_url: String,
    upload_params: HashMap<String, String>,
}

/// Step 2: POST multipart form data to `upload_url` without an Authorization header.
/// Strips the `while(1);` anti-CSRF prefix from the response body if present.
pub(crate) async fn execute_upload(
    client: &Client,
    upload_url: &str,
    upload_params: HashMap<String, String>,
    filename: String,
    content_type: String,
    data: Vec<u8>,
) -> Result<File> {
    // Build multipart form: upload_params first, then the file field (must be last).
    let mut form = Form::new();
    for (key, value) in upload_params {
        form = form.text(key, value);
    }

    // Build the file part, applying MIME type if valid.
    let file_part = Part::bytes(data.clone()).file_name(filename.clone());
    let file_part = file_part
        .mime_str(&content_type)
        .unwrap_or_else(|_| Part::bytes(data).file_name(filename));

    form = form.part("file", file_part);

    let resp = client.post(upload_url).multipart(form).send().await?;

    let body = resp.text().await?;

    // Canvas sometimes prefixes its JSON with `while(1);` to prevent CSRF via JSON hijacking.
    let json_str = body
        .strip_prefix("while(1);")
        .unwrap_or(&body);

    let file: File = serde_json::from_str(json_str)?;
    Ok(file)
}

/// Full two-step upload: POST metadata to Canvas to get upload intent, then upload the file.
pub(crate) async fn initiate_and_upload(
    requester: &crate::http::Requester,
    endpoint: &str,
    request: UploadRequest,
    data: Vec<u8>,
) -> Result<File> {
    // Serialize UploadRequest as flat params (no bracket notation) for Canvas step 1.
    let params = crate::params::flatten_params(&serde_json::to_value(&request).unwrap());

    let intent: UploadIntent = requester.post(endpoint, &params).await?;

    // Determine content_type for the multipart part.
    let content_type = request
        .content_type
        .unwrap_or_else(|| "application/octet-stream".to_string());

    execute_upload(
        &requester.client,
        &intent.upload_url,
        intent.upload_params,
        request.name,
        content_type,
        data,
    )
    .await
}
