use std::collections::BTreeMap;
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

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug)]
struct FolderParsedResult {
    folder_path: PathBuf,
    image_count: usize,
    merged_ocr_markdown: String,
    ai_markdown: String,
    output_md_path: PathBuf,
    ai_duration_ms: u64,
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
    let root_folder_path = PathBuf::from(request.folder_path.trim());
    if !root_folder_path.is_dir() {
        return Err(AppError::Validation("请选择有效的本地文件夹".into()));
    }

    let prompt = request.prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(AppError::Validation("结构化提示词不能为空".into()));
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

    let mut all_image_paths = collect_image_paths(&root_folder_path)?;
    if all_image_paths.is_empty() {
        return Err(AppError::Validation("文件夹中未找到可处理图片".into()));
    }
    all_image_paths.sort();

    let grouped = group_image_paths_by_parent(all_image_paths, &root_folder_path);
    let total_images: usize = grouped.values().map(Vec::len).sum();
    let total_folders = grouped.len();
    let total_steps = total_images + total_folders + 1;

    let asset_root = request
        .asset_root
        .map(PathBuf::from)
        .unwrap_or_else(default_asset_root);

    let started_at = Instant::now();
    let mut current_step = 0usize;
    let mut all_ocr_items = Vec::<FolderOcrItem>::new();
    let mut folder_results = Vec::<FolderParsedResult>::with_capacity(total_folders);
    let mut generated_files = Vec::<String>::new();

    for (folder_path, mut image_paths) in grouped {
        image_paths.sort();

        let mut folder_ocr_items = Vec::<FolderOcrItem>::with_capacity(image_paths.len());

        for (index, image_path) in image_paths.iter().enumerate() {
            current_step += 1;
            let display_name = image_path
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("image-{}", index + 1));

            emit_progress(
                app,
                FolderBatchProgressEvent {
                    stage: "ocr".into(),
                    current: current_step,
                    total: total_steps,
                    percent: progress_percent(current_step, total_steps),
                    current_image_name: Some(display_name.clone()),
                    message: format!("OCR {}/{}：{}", index + 1, image_paths.len(), display_name),
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

            let item = FolderOcrItem {
                order: index + 1,
                source_path: image_path.display().to_string(),
                display_name,
                text: ocr_text,
                duration_ms,
            };
            folder_ocr_items.push(item.clone());
            all_ocr_items.push(item);
        }

        current_step += 1;
        emit_progress(
            app,
            FolderBatchProgressEvent {
                stage: "ai".into(),
                current: current_step,
                total: total_steps,
                percent: progress_percent(current_step, total_steps),
                current_image_name: None,
                message: format!(
                    "正在结构化目录：{}",
                    relative_path_label(&root_folder_path, &folder_path)
                ),
                level: "info".into(),
            },
        );

        let folder_merged_ocr = build_merged_ocr_markdown(&folder_ocr_items);
        let mut deepseek_request = DeepSeekRequest::new(prompt.clone(), folder_merged_ocr.clone());
        if let Some(model) = request.model.clone() {
            deepseek_request = deepseek_request.with_model(model);
        }
        if let Some(api_url) = request.api_url.clone() {
            deepseek_request = deepseek_request.with_api_url(api_url);
        }

        let ai_started_at = Instant::now();
        let ai_result = ai_service::structure_text_with_deepseek(deepseek_request).await?;
        let ai_duration_ms = ai_started_at.elapsed().as_millis() as u64;

        let folder_name = folder_path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "folder".into());
        let folder_output_md_path = folder_path.join(format!("{folder_name}_parsed.md"));
        let folder_md_content = build_folder_markdown(
            &folder_path,
            &root_folder_path,
            &prompt,
            &folder_merged_ocr,
            &ai_result.markdown,
        );
        tokio::fs::write(&folder_output_md_path, folder_md_content).await?;
        generated_files.push(folder_output_md_path.display().to_string());

        folder_results.push(FolderParsedResult {
            folder_path,
            image_count: folder_ocr_items.len(),
            merged_ocr_markdown: folder_merged_ocr,
            ai_markdown: ai_result.markdown,
            output_md_path: folder_output_md_path,
            ai_duration_ms,
        });
    }

    current_step += 1;
    emit_progress(
        app,
        FolderBatchProgressEvent {
            stage: "ai".into(),
            current: current_step,
            total: total_steps,
            percent: progress_percent(current_step, total_steps),
            current_image_name: None,
            message: "正在生成根目录总 Markdown...".into(),
            level: "info".into(),
        },
    );

    let root_name = root_folder_path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "root".into());
    let root_summary_md_path = root_folder_path.join(format!("{root_name}_all_results.md"));
    let root_summary_content = build_root_summary_markdown(
        &root_folder_path,
        &prompt,
        &folder_results,
        total_images,
    );
    tokio::fs::write(&root_summary_md_path, root_summary_content).await?;
    generated_files.push(root_summary_md_path.display().to_string());

    emit_progress(
        app,
        FolderBatchProgressEvent {
            stage: "done".into(),
            current: total_steps,
            total: total_steps,
            percent: 100,
            current_image_name: None,
            message: format!(
                "批处理完成：{} 个目录结果 + 1 个根目录总文件。",
                total_folders
            ),
            level: "success".into(),
        },
    );

    let merged_ocr_text = build_all_folders_ocr_markdown(&root_folder_path, &folder_results);
    let ai_markdown = build_all_folders_ai_markdown(&root_folder_path, &folder_results);
    let ai_duration_ms = folder_results
        .iter()
        .map(|item| item.ai_duration_ms)
        .sum::<u64>();

    Ok(ProcessImageFolderResponse {
        folder_path: root_folder_path.display().to_string(),
        image_count: total_images,
        ocr_items: all_ocr_items,
        merged_ocr_text,
        ai_markdown,
        consolidated_md_path: root_summary_md_path.display().to_string(),
        generated_files,
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

fn group_image_paths_by_parent(
    image_paths: Vec<PathBuf>,
    root_folder_path: &Path,
) -> BTreeMap<PathBuf, Vec<PathBuf>> {
    let mut grouped = BTreeMap::<PathBuf, Vec<PathBuf>>::new();
    for path in image_paths {
        let parent = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| root_folder_path.to_path_buf());
        grouped.entry(parent).or_default().push(path);
    }
    grouped
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

fn build_folder_markdown(
    folder_path: &Path,
    root_folder_path: &Path,
    prompt: &str,
    merged_ocr_markdown: &str,
    ai_markdown: &str,
) -> String {
    let mut sections = Vec::<String>::new();
    sections.push("# 目录解析结果".into());
    sections.push(format!(
        "- 目录：{}",
        relative_path_label(root_folder_path, folder_path)
    ));
    sections.push(String::new());
    sections.push("## 结构化提示词".into());
    sections.push(prompt.trim().to_string());
    sections.push(String::new());
    sections.push("## OCR 识别结果".into());
    sections.push(merged_ocr_markdown.trim().to_string());
    sections.push(String::new());
    sections.push("## AI 结构化结果".into());
    sections.push(ai_markdown.trim().to_string());
    sections.join("\n")
}

fn build_root_summary_markdown(
    root_folder_path: &Path,
    prompt: &str,
    folder_results: &[FolderParsedResult],
    total_images: usize,
) -> String {
    let mut sections = Vec::<String>::new();
    sections.push("# 根目录总解析结果".into());
    sections.push(format!("- 根目录：{}", root_folder_path.display()));
    sections.push(format!("- 目录数量：{}", folder_results.len()));
    sections.push(format!("- 图片总数：{}", total_images));
    sections.push(String::new());
    sections.push("## 结构化提示词".into());
    sections.push(prompt.trim().to_string());
    sections.push(String::new());

    for (idx, result) in folder_results.iter().enumerate() {
        sections.push(format!(
            "## 目录 {}/{}：{}",
            idx + 1,
            folder_results.len(),
            relative_path_label(root_folder_path, &result.folder_path)
        ));
        sections.push(format!("- 图片数：{}", result.image_count));
        sections.push(format!("- 目录结果文件：{}", result.output_md_path.display()));
        sections.push(String::new());
        sections.push("### OCR 识别结果".into());
        sections.push(result.merged_ocr_markdown.trim().to_string());
        sections.push(String::new());
        sections.push("### AI 结构化结果".into());
        sections.push(result.ai_markdown.trim().to_string());
        sections.push(String::new());
    }

    sections.join("\n")
}

fn build_all_folders_ocr_markdown(root_folder_path: &Path, folder_results: &[FolderParsedResult]) -> String {
    folder_results
        .iter()
        .map(|result| {
            format!(
                "## 目录：{}\n\n{}",
                relative_path_label(root_folder_path, &result.folder_path),
                result.merged_ocr_markdown.trim()
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn build_all_folders_ai_markdown(root_folder_path: &Path, folder_results: &[FolderParsedResult]) -> String {
    folder_results
        .iter()
        .map(|result| {
            format!(
                "## 目录：{}\n\n{}",
                relative_path_label(root_folder_path, &result.folder_path),
                result.ai_markdown.trim()
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn relative_path_label(root: &Path, target: &Path) -> String {
    match target.strip_prefix(root) {
        Ok(relative) if relative.as_os_str().is_empty() => "./".into(),
        Ok(relative) => relative.display().to_string(),
        Err(_) => target.display().to_string(),
    }
}

fn progress_percent(current_step: usize, total_steps: usize) -> u8 {
    if total_steps == 0 {
        return 0;
    }
    ((current_step * 100) / total_steps).min(100) as u8
}
