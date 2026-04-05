use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::services::export_service::normalize_export_filename as normalize_file_name;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizeExportFilenameRequest {
    pub base_name: String,
    pub extension: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizeExportFilenameResponse {
    pub file_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveTextExportRequest {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveTextExportResponse {
    pub path: String,
}

#[tauri::command]
pub fn normalize_export_filename(
    request: NormalizeExportFilenameRequest,
) -> AppResult<NormalizeExportFilenameResponse> {
    Ok(NormalizeExportFilenameResponse {
        file_name: normalize_file_name(&request.base_name, &request.extension)?,
    })
}

#[tauri::command]
pub fn save_text_export(request: SaveTextExportRequest) -> AppResult<SaveTextExportResponse> {
    std::fs::write(&request.path, request.content)?;
    Ok(SaveTextExportResponse { path: request.path })
}
