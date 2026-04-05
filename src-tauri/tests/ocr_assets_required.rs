#[test]
fn missing_assets_return_error() {
    let result = smartocr_pro::services::ocr_service::validate_asset_layout(std::path::Path::new(
        "missing",
    ));

    assert!(result.is_err());
}

#[test]
fn portable_asset_root_prefers_exe_neighbor_resources() {
    let exe_dir = std::path::Path::new("C:/portable/SmartOCR Pro");
    let manifest_dir = std::path::Path::new("D:/repo/src-tauri");

    let resolved = smartocr_pro::services::ocr_service::resolve_default_asset_root(exe_dir, manifest_dir);

    assert_eq!(resolved, exe_dir.join("resources").join("ocr"));
}

#[test]
fn portable_asset_root_falls_back_to_manifest_resources() {
    let exe_dir = std::path::Path::new("C:/portable/SmartOCR Pro");
    let manifest_dir = std::path::Path::new("D:/repo/src-tauri");

    let resolved =
        smartocr_pro::services::ocr_service::resolve_fallback_asset_root(exe_dir, manifest_dir);

    assert_eq!(resolved, manifest_dir.join("resources").join("ocr"));
}

#[test]
fn missing_required_model_file_returns_error() {
    let root = make_temp_asset_root("missing-model-file");
    std::fs::create_dir_all(root.join("runtime")).unwrap();
    std::fs::create_dir_all(root.join("models")).unwrap();
    std::fs::write(root.join("models/ch_PP-OCRv5_mobile_det.onnx"), b"det").unwrap();
    std::fs::write(root.join("models/ch_ppocr_mobile_v2.0_cls_infer.onnx"), b"cls").unwrap();

    let result = smartocr_pro::services::ocr_service::resolve_model_paths(&root);

    assert!(result.is_err());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn resolves_required_model_files() {
    let root = make_temp_asset_root("resolve-model-files");
    std::fs::create_dir_all(root.join("runtime")).unwrap();
    std::fs::create_dir_all(root.join("models")).unwrap();
    std::fs::write(root.join("models/ch_PP-OCRv5_mobile_det.onnx"), b"det").unwrap();
    std::fs::write(root.join("models/ch_ppocr_mobile_v2.0_cls_infer.onnx"), b"cls").unwrap();
    std::fs::write(root.join("models/ch_PP-OCRv5_rec_mobile_infer.onnx"), b"rec").unwrap();

    let result = smartocr_pro::services::ocr_service::resolve_model_paths(&root).unwrap();

    assert_eq!(
        result.det,
        root.join("models").join("ch_PP-OCRv5_mobile_det.onnx")
    );
    assert_eq!(
        result.cls,
        root.join("models").join("ch_ppocr_mobile_v2.0_cls_infer.onnx")
    );
    assert_eq!(
        result.rec,
        root.join("models").join("ch_PP-OCRv5_rec_mobile_infer.onnx")
    );
    let _ = std::fs::remove_dir_all(root);
}

fn make_temp_asset_root(suffix: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "smartocr-pro-tests-{}-{}",
        suffix,
        std::process::id()
    ));

    let _ = std::fs::remove_dir_all(&root);
    root
}
