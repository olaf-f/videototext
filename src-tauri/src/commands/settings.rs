use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::models::AppSettings;
use crate::services::settings_service::SettingsService;

#[tauri::command]
pub fn load_settings() -> AppResult<AppSettings> {
    SettingsService::new()?.load()
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> AppResult<()> {
    SettingsService::new()?.save(&settings)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDeepSeekApiKeyRequest {
    pub api_key: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDeepSeekApiKeyResponse {
    pub saved: bool,
}

#[tauri::command]
pub fn save_deepseek_api_key(
    request: SaveDeepSeekApiKeyRequest,
) -> AppResult<SaveDeepSeekApiKeyResponse> {
    SettingsService::new()?.save_deepseek_api_key(&request.api_key)?;
    Ok(SaveDeepSeekApiKeyResponse { saved: true })
}
