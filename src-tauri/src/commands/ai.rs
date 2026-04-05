use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::services::ai_service::{structure_text_with_deepseek as request_deepseek_markdown, DeepSeekRequest};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructureTextRequest {
    pub prompt: String,
    pub ocr_text: String,
    pub model: Option<String>,
    pub api_url: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructureTextResponse {
    pub markdown: String,
}

#[tauri::command]
pub async fn structure_text_with_deepseek(
    request: StructureTextRequest,
) -> AppResult<StructureTextResponse> {
    let mut deepseek_request = DeepSeekRequest::new(request.prompt, request.ocr_text);

    if let Some(model) = request.model {
        deepseek_request = deepseek_request.with_model(model);
    }

    if let Some(api_url) = request.api_url {
        deepseek_request = deepseek_request.with_api_url(api_url);
    }

    let response = request_deepseek_markdown(deepseek_request).await?;
    Ok(StructureTextResponse {
        markdown: response.markdown,
    })
}
