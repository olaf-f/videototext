import type { ExportExtension } from './export'

export type ImageSourceType = 'clipboard' | 'file' | 'url'
export type BusyState = 'ai' | 'export' | 'import' | 'ocr' | 'pipeline' | 'settings' | null

export type ActiveImageSource = {
  id: string
  sourceType: ImageSourceType
  displayName: string
  mimeType: string
  previewUrl: string
  byteLength: number
  imageBytes: number[]
}

export type OcrResultItem = {
  order: number
  sourceId: string
  displayName: string
  text: string
  durationMs: number
}

export type OcrResult = {
  items: OcrResultItem[]
  text: string
  durationMs: number
}

export type AiResult = {
  markdown: string
  durationMs: number
  sourceIds: string[]
}

export type AppSettings = {
  deepseekApiKeySaved: boolean
  defaultPrompt: string
  webPortalUrl: string
}

export type SaveSettingsPayload = {
  deepseekApiKey: string
  defaultPrompt: string
  webPortalUrl: string
}

export type AccumulatorExportRequest = {
  content: string
  extension: ExportExtension
}
