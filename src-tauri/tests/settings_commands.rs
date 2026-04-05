use smartocr_pro::commands::settings::{load_settings, save_settings};
use smartocr_pro::models::AppSettings;

#[test]
fn save_settings_command_accepts_full_settings_payload() {
    let temp_dir = std::env::temp_dir().join(format!(
        "smartocr-pro-settings-command-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::env::set_var("SMARTOCR_PRO_SETTINGS_DIR", &temp_dir);

    let settings = AppSettings {
        deepseek_api_key_saved: false,
        default_prompt: "Prompt".into(),
        web_portal_url: "https://example.com/portal".into(),
    };

    let result = save_settings(settings);

    assert!(result.is_ok(), "expected save_settings to succeed: {result:?}");
    let loaded = load_settings().unwrap();
    assert_eq!(loaded.web_portal_url, "https://example.com/portal");

    std::env::remove_var("SMARTOCR_PRO_SETTINGS_DIR");
    std::fs::remove_dir_all(temp_dir).unwrap();
}
