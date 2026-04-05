use smartocr_pro::models::AppSettings;
use smartocr_pro::services::settings_service::{
    load_settings_for_test, save_settings_for_test, SettingsService,
};

#[test]
fn settings_roundtrip() {
    let temp_dir = std::env::temp_dir().join(format!(
        "smartocr-pro-settings-roundtrip-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    std::fs::create_dir_all(&temp_dir).unwrap();

    let expected = AppSettings {
        deepseek_api_key_saved: true,
        default_prompt: "Extract all visible text".into(),
        web_portal_url: "https://example.com/web-ocr".into(),
    };

    save_settings_for_test(temp_dir.clone(), &expected).unwrap();
    let loaded = load_settings_for_test(temp_dir.clone()).unwrap();

    assert!(!loaded.deepseek_api_key_saved);
    assert_eq!(loaded.default_prompt, expected.default_prompt);
    assert_eq!(loaded.web_portal_url, expected.web_portal_url);

    std::fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn settings_service_uses_provided_test_directory() {
    let temp_dir = std::env::temp_dir().join(format!(
        "smartocr-pro-settings-dir-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let service = SettingsService::new_for_tests(temp_dir.clone());
    let settings_path = service.settings_path().to_path_buf();

    assert!(settings_path.starts_with(&temp_dir));
    assert!(settings_path.ends_with("settings.json"));
}
