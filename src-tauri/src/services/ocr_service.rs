use std::path::{Path, PathBuf};
use std::sync::Once;

use paddle_ocr_rs::ocr_lite::OcrLite;

use crate::error::{AppError, AppResult};
use crate::services::image_io::decode_image;

const OCR_ASSET_DIR: &str = "resources/ocr";
const DET_MODEL_FILE: &str = "ch_PP-OCRv5_mobile_det.onnx";
const CLS_MODEL_FILE: &str = "ch_ppocr_mobile_v2.0_cls_infer.onnx";
const REC_MODEL_FILE: &str = "ch_PP-OCRv5_rec_mobile_infer.onnx";
static OCR_RUNTIME_PATH_ONCE: Once = Once::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcrModelPaths {
    pub det: PathBuf,
    pub cls: PathBuf,
    pub rec: PathBuf,
}

pub fn default_asset_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    match std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
    {
        Some(exe_dir) => {
            let portable_root = resolve_default_asset_root(&exe_dir, manifest_dir);
            if portable_root.is_dir() {
                portable_root
            } else {
                resolve_fallback_asset_root(&exe_dir, manifest_dir)
            }
        }
        None => resolve_fallback_asset_root(manifest_dir, manifest_dir),
    }
}

pub fn resolve_default_asset_root(exe_dir: &Path, _manifest_dir: &Path) -> PathBuf {
    exe_dir.join(OCR_ASSET_DIR)
}

pub fn resolve_fallback_asset_root(_exe_dir: &Path, manifest_dir: &Path) -> PathBuf {
    manifest_dir.join(OCR_ASSET_DIR)
}

pub fn validate_asset_layout(root: &Path) -> AppResult<()> {
    validate_required_dir(root, "runtime")?;
    validate_required_dir(root, "models")?;
    Ok(())
}

pub fn run_ocr_blocking(asset_root: &Path, image_bytes: &[u8]) -> AppResult<String> {
    validate_asset_layout(asset_root)?;
    configure_runtime_library_path(asset_root);
    let model_paths = resolve_model_paths(asset_root)?;
    let image = decode_image(image_bytes)?.to_rgb8();
    let mut ocr = OcrLite::new();

    ocr.init_models(
        &model_paths.det.to_string_lossy(),
        &model_paths.cls.to_string_lossy(),
        &model_paths.rec.to_string_lossy(),
        2,
    )
    .map_err(|error| AppError::OcrRuntime(error.to_string()))?;

    let result = ocr
        .detect(&image, 50, 1024, 0.5, 0.3, 1.6, true, false)
        .map_err(|error| AppError::OcrRuntime(error.to_string()))?;

    Ok(result
        .text_blocks
        .into_iter()
        .map(|block| block.text.trim().to_string())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n"))
}

pub fn resolve_model_paths(root: &Path) -> AppResult<OcrModelPaths> {
    validate_asset_layout(root)?;

    Ok(OcrModelPaths {
        det: validate_required_file(root, "detector model", &format!("models/{DET_MODEL_FILE}"))?,
        cls: validate_required_file(
            root,
            "classifier model",
            &format!("models/{CLS_MODEL_FILE}"),
        )?,
        rec: validate_required_file(
            root,
            "recognizer model",
            &format!("models/{REC_MODEL_FILE}"),
        )?,
    })
}

fn validate_required_dir(root: &Path, name: &'static str) -> AppResult<()> {
    let path = root.join(name);
    if path.is_dir() {
        return Ok(());
    }

    Err(AppError::MissingOcrAssetDirectory {
        kind: name,
        path: path.display().to_string(),
    })
}

fn validate_required_file(root: &Path, kind: &'static str, relative_path: &str) -> AppResult<PathBuf> {
    let path = root.join(relative_path);
    if path.is_file() {
        return Ok(path);
    }

    Err(AppError::MissingOcrAssetFile {
        kind,
        path: path.display().to_string(),
    })
}

fn configure_runtime_library_path(asset_root: &Path) {
    let runtime_dir = asset_root.join("runtime");

    OCR_RUNTIME_PATH_ONCE.call_once(|| {
        if !runtime_dir.is_dir() {
            return;
        }

        let existing = std::env::var_os("PATH").unwrap_or_default();
        let mut paths = std::env::split_paths(&existing).collect::<Vec<_>>();
        if paths.iter().any(|path| path == &runtime_dir) {
            return;
        }

        paths.insert(0, runtime_dir);
        if let Ok(joined) = std::env::join_paths(paths) {
            std::env::set_var("PATH", joined);
        }
    });
}
