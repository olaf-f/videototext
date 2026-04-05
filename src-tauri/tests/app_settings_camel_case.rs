use serde_json::json;
use smartocr_pro::models::AppSettings;

#[test]
fn app_settings_serializes_with_camel_case_keys() {
    let settings = AppSettings {
        deepseek_api_key_saved: true,
        default_prompt: "Prompt".into(),
        web_portal_url: "https://example.com/web-ocr".into(),
    };

    let value = serde_json::to_value(&settings).expect("serialize AppSettings");

    assert_eq!(value["deepseekApiKeySaved"], json!(true));
    assert_eq!(value["defaultPrompt"], json!("Prompt"));
    assert_eq!(value["webPortalUrl"], json!("https://example.com/web-ocr"));
    assert!(value.get("deepseek_api_key_saved").is_none());
}

#[test]
fn app_settings_deserializes_from_camel_case_keys() {
    let value = json!({
        "deepseekApiKeySaved": true,
        "defaultPrompt": "Prompt",
        "webPortalUrl": "https://example.com/web-ocr"
    });

    let settings: AppSettings = serde_json::from_value(value).expect("deserialize AppSettings");
    assert!(settings.deepseek_api_key_saved);
    assert_eq!(settings.default_prompt, "Prompt");
    assert_eq!(settings.web_portal_url, "https://example.com/web-ocr");
}
