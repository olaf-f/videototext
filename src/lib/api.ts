import { invoke } from '@tauri-apps/api/core'

import type { AppSettings } from './types'

type RunOcrRequest = {
  assetRoot?: string
  imageBytes: number[]
}

type RunOcrResponse = {
  text: string
}

type StructureTextRequest = {
  apiUrl?: string
  model?: string
  ocrText: string
  prompt: string
}

type StructureTextResponse = {
  markdown: string
}

type NormalizeExportFilenameRequest = {
  baseName: string
  extension: string
}

type NormalizeExportFilenameResponse = {
  fileName: string
}

type SaveTextExportRequest = {
  path: string
  content: string
}

type SaveTextExportResponse = {
  path: string
}

type ImportImageFromUrlRequest = {
  url: string
}

type ImportImageFromUrlResponse = {
  url: string
  displayName: string | null
  contentType: string
  imageBytes: number[]
}

type ProcessImageFolderRequest = {
  folderPath: string
  prompt: string
  assetRoot?: string
  model?: string
  apiUrl?: string
}

export type FolderBatchProgressEvent = {
  stage: 'scan' | 'ocr' | 'ai' | 'done' | 'error'
  current: number
  total: number
  percent: number
  currentImageName: string | null
  message: string
  level: 'info' | 'success' | 'error'
}

export type FolderBatchOcrItem = {
  order: number
  sourcePath: string
  displayName: string
  text: string
  durationMs: number
}

type ProcessImageFolderResponse = {
  folderPath: string
  imageCount: number
  ocrItems: FolderBatchOcrItem[]
  mergedOcrText: string
  aiMarkdown: string
  consolidatedMdPath: string
  generatedFiles: string[]
  totalDurationMs: number
  aiDurationMs: number
}

export async function loadSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('load_settings')
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  await invoke('save_settings', { settings })
}

export async function saveDeepseekApiKey(apiKey: string): Promise<void> {
  await invoke('save_deepseek_api_key', { request: { apiKey } })
}

export async function runOcr(request: RunOcrRequest): Promise<RunOcrResponse> {
  return invoke<RunOcrResponse>('run_ocr', { request })
}

export async function validateOcrAssets(assetRoot?: string): Promise<void> {
  await invoke('validate_ocr_assets', { assetRoot })
}

export async function structureTextWithDeepseek(
  request: StructureTextRequest,
): Promise<StructureTextResponse> {
  return invoke<StructureTextResponse>('structure_text_with_deepseek', { request })
}

export async function validateImageUrlContentType(
  url: string,
  contentType: string,
): Promise<void> {
  await invoke('validate_image_url_content_type', { contentType, url })
}

export async function importImageFromUrl(url: string): Promise<ImportImageFromUrlResponse> {
  const request: ImportImageFromUrlRequest = { url }
  return invoke<ImportImageFromUrlResponse>('import_image_from_url', { request })
}

export async function normalizeExportFilename(
  request: NormalizeExportFilenameRequest,
): Promise<NormalizeExportFilenameResponse> {
  return invoke<NormalizeExportFilenameResponse>('normalize_export_filename', { request })
}

export async function saveTextExport(
  request: SaveTextExportRequest,
): Promise<SaveTextExportResponse> {
  return invoke<SaveTextExportResponse>('save_text_export', { request })
}

export async function processImageFolder(
  request: ProcessImageFolderRequest,
): Promise<ProcessImageFolderResponse> {
  return invoke<ProcessImageFolderResponse>('process_image_folder', { request })
}
