use serde::Deserialize;

use crate::error::{AppError, AppResult};

const DEFAULT_DEEPSEEK_MODEL: &str = "deepseek-chat";
const DEFAULT_DEEPSEEK_API_URL: &str = "https://api.deepseek.com/chat/completions";
const KEYRING_SERVICE: &str = "smartocr-pro";
const KEYRING_USERNAME: &str = "deepseek-api-key";

#[derive(Debug, Clone)]
pub struct DeepSeekRequest {
    prompt: String,
    ocr_text: String,
    model: String,
    api_url: String,
}

impl DeepSeekRequest {
    pub fn new(prompt: impl Into<String>, ocr_text: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            ocr_text: ocr_text.into(),
            model: DEFAULT_DEEPSEEK_MODEL.into(),
            api_url: DEFAULT_DEEPSEEK_API_URL.into(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        let model = model.into();
        if !model.trim().is_empty() {
            self.model = model;
        }
        self
    }

    pub fn with_api_url(mut self, api_url: impl Into<String>) -> Self {
        let api_url = api_url.into();
        if !api_url.trim().is_empty() {
            self.api_url = api_url;
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeepSeekResult {
    pub markdown: String,
}

pub fn build_deepseek_payload(prompt: &str, ocr_text: &str) -> serde_json::Value {
    serde_json::json!({
        "model": DEFAULT_DEEPSEEK_MODEL,
        "messages": [
            {
                "role": "user",
                "content": format!("Prompt:\n{}\n\nOCR Text:\n{}", prompt.trim(), ocr_text.trim())
            }
        ],
        "stream": false
    })
}

pub async fn structure_text_with_deepseek(request: DeepSeekRequest) -> AppResult<DeepSeekResult> {
    let api_key = load_deepseek_api_key()?;
    let payload = serde_json::json!({
        "model": request.model,
        "messages": [
            {
                "role": "user",
                "content": format!("Prompt:\n{}\n\nOCR Text:\n{}", request.prompt.trim(), request.ocr_text.trim())
            }
        ],
        "stream": false
    });

    let response = reqwest::Client::new()
        .post(&request.api_url)
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|error| AppError::DeepSeekRequest(error.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "unable to read DeepSeek error body".into());
        return Err(AppError::DeepSeekApi(format!("{status}: {body}")));
    }

    let completion: DeepSeekCompletionResponse = response
        .json()
        .await
        .map_err(|error| AppError::DeepSeekRequest(error.to_string()))?;
    let markdown = completion
        .choices
        .into_iter()
        .find_map(|choice| {
            let content = choice.message.content.trim().to_string();
            (!content.is_empty()).then_some(content)
        })
        .ok_or(AppError::DeepSeekEmptyResponse)?;

    Ok(DeepSeekResult { markdown })
}

fn load_deepseek_api_key() -> AppResult<String> {
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME) {
        if let Ok(value) = entry.get_password() {
            let value = value.trim().to_string();
            if !value.is_empty() {
                return Ok(value);
            }
        }
    }

    ["SMARTOCR_PRO_DEEPSEEK_API_KEY", "DEEPSEEK_API_KEY"]
        .into_iter()
        .filter_map(|name| std::env::var(name).ok())
        .map(|value| value.trim().to_string())
        .find(|value| !value.is_empty())
        .ok_or(AppError::DeepSeekApiKeyUnavailable)
}

pub fn has_deepseek_api_key() -> bool {
    load_deepseek_api_key().is_ok()
}

pub fn save_deepseek_api_key(api_key: &str) -> AppResult<()> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("DeepSeek API Key 不能为空".into()));
    }

    // Ensure the key is immediately available in current app process.
    std::env::set_var("SMARTOCR_PRO_DEEPSEEK_API_KEY", trimmed);

    // Best effort keyring persistence; fallback remains settings cache + process env.
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME) {
        let _ = entry.set_password(trimmed);
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct DeepSeekCompletionResponse {
    choices: Vec<DeepSeekChoice>,
}

#[derive(Debug, Deserialize)]
struct DeepSeekChoice {
    message: DeepSeekChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct DeepSeekChoiceMessage {
    content: String,
}
