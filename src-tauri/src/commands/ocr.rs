use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::services::ocr_service::{default_asset_root, run_ocr_blocking, validate_asset_layout};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunOcrRequest {
    pub image_bytes: Vec<u8>,
    pub asset_root: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunOcrResponse {
    pub text: String,
}

#[tauri::command]
pub fn validate_ocr_assets(asset_root: Option<String>) -> AppResult<()> {
    let root = asset_root.map(PathBuf::from).unwrap_or_else(default_asset_root);
    validate_asset_layout(&root)
}

#[tauri::command]
pub async fn run_ocr(request: RunOcrRequest) -> AppResult<RunOcrResponse> {
    let asset_root = request
        .asset_root
        .map(PathBuf::from)
        .unwrap_or_else(default_asset_root);
    let image_bytes = request.image_bytes;

    tauri::async_runtime::spawn_blocking(move || {
        let text = run_ocr_blocking(&asset_root, &image_bytes)?;
        Ok(RunOcrResponse { text })
    })
    .await
    .map_err(|error| AppError::OcrTaskJoin(error.to_string()))?
}
