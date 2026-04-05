use smartocr_pro::commands::ocr::validate_ocr_assets;

#[test]
fn validate_ocr_assets_command_rejects_missing_root() {
    let result = validate_ocr_assets(Some("missing".into()));

    assert!(result.is_err(), "expected missing OCR assets to fail");
}
