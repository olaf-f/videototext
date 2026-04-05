#[test]
fn export_filename_uses_safe_ascii() {
    let value = smartocr_pro::services::export_service::safe_export_name("AI Result 2026/04/03");

    assert_eq!(value, "ai-result-2026-04-03");
}
