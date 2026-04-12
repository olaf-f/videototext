use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tauri::Emitter;
use walkdir::WalkDir;

use crate::error::{AppError, AppResult};
use crate::services::ai_service::{self, DeepSeekRequest};
use crate::services::ocr_service::{default_asset_root, run_ocr_blocking};

const PROGRESS_EVENT_NAME: &str = "folder-batch-progress";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessImageFolderRequest {
    pub folder_path: String,
    pub prompt: String,
    pub asset_root: Option<String>,
    pub model: Option<String>,
    pub api_url: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderOcrItem {
    pub order: usize,
    pub source_path: String,
    pub display_name: String,
    pub text: String,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessImageFolderResponse {
    pub folder_path: String,
    pub image_count: usize,
    pub ocr_items: Vec<FolderOcrItem>,
    pub merged_ocr_text: String,
    pub ai_markdown: String,
    pub consolidated_md_path: String,
    pub generated_files: Vec<String>,
    pub total_duration_ms: u64,
    pub ai_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderBatchProgressEvent {
    pub stage: String,
    pub current: usize,
    pub total: usize,
    pub percent: u8,
    pub current_image_name: Option<String>,
    pub message: String,
    pub level: String,
}

#[tauri::command]
pub async fn process_image_folder(
    app: tauri::AppHandle,
    request: ProcessImageFolderRequest,
) -> AppResult<ProcessImageFolderResponse> {
    let result = process_image_folder_impl(&app, request).await;
    if let Err(error) = &result {
        emit_progress(
            &app,
            FolderBatchProgressEvent {
                stage: "error".into(),
                current: 0,
                total: 0,
                percent: 0,
                current_image_name: None,
                message: error.to_string(),
                level: "error".into(),
            },
        );
    }
    result
}

async fn process_image_folder_impl(
    app: &tauri::AppHandle,
    request: ProcessImageFolderRequest,
) -> AppResult<ProcessImageFolderResponse> {
    let folder_path = PathBuf::from(request.folder_path.trim());
    if !folder_path.is_dir() {
        return Err(AppError::Validation("请选择有效的本地文件夹".into()));
    }

    emit_progress(
        app,
        FolderBatchProgressEvent {
            stage: "scan".into(),
            current: 0,
            total: 0,
            percent: 0,
            current_image_name: None,
            message: "正在递归扫描图片文件...".into(),
            level: "info".into(),
        },
    );

    let mut image_paths = collect_image_paths(&folder_path)?;
    if image_paths.is_empty() {
        return Err(AppError::Validation("文件夹中未找到可处理图片".into()));
    }
    image_paths.sort();

    let prompt = request.prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(AppError::Validation("结构化提示词不能为空".into()));
    }

    let asset_root = request
        .asset_root
        .map(PathBuf::from)
        .unwrap_or_else(default_asset_root);
    let total_images = image_paths.len();
    let total_steps = total_images + 2;
    let mut ocr_items = Vec::<FolderOcrItem>::with_capacity(total_images);
    let started_at = Instant::now();

    for (index, image_path) in image_paths.iter().enumerate() {
        let order = index + 1;
        let display_name = image_path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("image-{order}"));

        emit_progress(
            app,
            FolderBatchProgressEvent {
                stage: "ocr".into(),
                current: order,
                total: total_images,
                percent: progress_percent(order, total_steps),
                current_image_name: Some(display_name.clone()),
                message: format!("OCR {order}/{total_images}: {display_name}"),
                level: "info".into(),
            },
        );

        let image_bytes = tokio::fs::read(image_path).await?;
        let ocr_started_at = Instant::now();
        let ocr_asset_root = asset_root.clone();
        let ocr_text = tauri::async_runtime::spawn_blocking(move || {
            run_ocr_blocking(&ocr_asset_root, &image_bytes)
        })
        .await
        .map_err(|error| AppError::OcrTaskJoin(error.to_string()))??;
        let duration_ms = ocr_started_at.elapsed().as_millis() as u64;

        ocr_items.push(FolderOcrItem {
            order,
            source_path: image_path.display().to_string(),
            display_name,
            text: ocr_text,
            duration_ms,
        });
    }

    let merged_ocr_text = build_merged_ocr_markdown(&ocr_items);

    emit_progress(
        app,
        FolderBatchProgressEvent {
            stage: "ai".into(),
            current: total_images + 1,
            total: total_steps,
            percent: progress_percent(total_images + 1, total_steps),
            current_image_name: None,
            message: "正在执行全量 AI 结构化分析...".into(),
            level: "info".into(),
        },
    );

    let mut deepseek_request = DeepSeekRequest::new(prompt.clone(), merged_ocr_text.clone());
    if let Some(model) = request.model {
        deepseek_request = deepseek_request.with_model(model);
    }
    if let Some(api_url) = request.api_url {
        deepseek_request = deepseek_request.with_api_url(api_url);
    }

    let ai_started_at = Instant::now();
    let ai_result = ai_service::structure_text_with_deepseek(deepseek_request).await?;
    let ai_duration_ms = ai_started_at.elapsed().as_millis() as u64;
    let ai_markdown = ai_result.markdown;

    let folder_name = folder_path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "images".into());

    let consolidated_md = build_consolidated_markdown(
        &folder_path,
        &prompt,
        &ocr_items,
        &merged_ocr_text,
        &ai_markdown,
    );

    let consolidated_md_path = folder_path.join(format!("{folder_name}_all_results.md"));
    tokio::fs::write(&consolidated_md_path, consolidated_md).await?;

    emit_progress(
        app,
        FolderBatchProgressEvent {
            stage: "done".into(),
            current: total_steps,
            total: total_steps,
            percent: 100,
            current_image_name: None,
            message: format!("批处理完成，共处理 {total_images} 张图片，已输出 1 个 Markdown 文件。"),
            level: "success".into(),
        },
    );

    Ok(ProcessImageFolderResponse {
        folder_path: folder_path.display().to_string(),
        image_count: total_images,
        ocr_items,
        merged_ocr_text,
        ai_markdown,
        consolidated_md_path: consolidated_md_path.display().to_string(),
        generated_files: vec![consolidated_md_path.display().to_string()],
        total_duration_ms: started_at.elapsed().as_millis() as u64,
        ai_duration_ms,
    })
}

fn emit_progress(app: &tauri::AppHandle, payload: FolderBatchProgressEvent) {
    let _ = app.emit(PROGRESS_EVENT_NAME, payload);
}

fn collect_image_paths(root: &Path) -> AppResult<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in WalkDir::new(root).follow_links(false).into_iter() {
        let entry = entry.map_err(|error| AppError::Validation(error.to_string()))?;
        if !entry.file_type().is_file() {
            continue;
        }

        if is_supported_image_path(entry.path()) {
            paths.push(entry.path().to_path_buf());
        }
    }
    Ok(paths)
}

fn is_supported_image_path(path: &Path) -> bool {
    let extension = path
        .extension()
        .map(|value| value.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    matches!(
        extension.as_str(),
        "jpg" | "jpeg" | "png" | "bmp" | "webp" | "tif" | "tiff" | "gif"
    )
}

fn build_merged_ocr_markdown(items: &[FolderOcrItem]) -> String {
    items
        .iter()
        .map(|item| {
            let body = item.text.trim();
            if body.is_empty() {
                format!(
                    "## 图片 {}/{}：{}\n\n（未识别到有效文本）",
                    item.order,
                    items.len(),
                    item.display_name
                )
            } else {
                format!(
                    "## 图片 {}/{}：{}\n\n{}",
                    item.order,
                    items.len(),
                    item.display_name,
                    body
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn build_consolidated_markdown(
    folder_path: &Path,
    prompt: &str,
    items: &[FolderOcrItem],
    merged_ocr_text: &str,
    ai_markdown: &str,
) -> String {
    let mut sections = Vec::<String>::new();

    sections.push("# 文件夹批处理结果".into());
    sections.push(format!("- 文件夹：{}", folder_path.display()));
    sections.push(format!("- 图片数量：{}", items.len()));
    sections.push(String::new());
    sections.push("## 结构化提示词".into());
    sections.push(prompt.trim().to_string());
    sections.push(String::new());
    sections.push("## OCR 识别结果（按顺序）".into());
    sections.push(merged_ocr_text.trim().to_string());
    sections.push(String::new());
    sections.push("## AI 结构化结果".into());
    sections.push(ai_markdown.trim().to_string());

    sections.join("\n")
}

fn progress_percent(current_step: usize, total_steps: usize) -> u8 {
    if total_steps == 0 {
        return 0;
    }
    ((current_step * 100) / total_steps).min(100) as u8
}
