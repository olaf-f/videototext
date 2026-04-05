use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub deepseek_api_key_saved: bool,
    pub default_prompt: String,
    pub web_portal_url: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            deepseek_api_key_saved: false,
            default_prompt: "请提取图像中的关键信息，并输出简洁的 Markdown 结构化摘要。".into(),
            web_portal_url: "https://example.com/web-ocr".into(),
        }
    }
}
