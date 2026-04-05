import { save } from '@tauri-apps/plugin-dialog'

import { normalizeExportFilename, saveTextExport } from './api'

export type ExportExtension = 'md' | 'txt'

const APP_EXPORT_FALLBACK = 'smartocr-pro'

export function appendAccumulatorSection(
  existing: string,
  title: string,
  body: string,
): string {
  const trimmedBody = body.trim()
  if (!trimmedBody) {
    return existing
  }

  const nextSection = `## ${title}\n\n${trimmedBody}`
  return existing.trim() ? `${existing.trim()}\n\n${nextSection}` : nextSection
}

export function inferExportBaseName(displayName: string): string {
  const trimmedName = displayName.trim()
  if (!trimmedName) {
    return APP_EXPORT_FALLBACK
  }

  return trimmedName.replace(/\.[a-z0-9]+$/i, '').trim() || APP_EXPORT_FALLBACK
}

export async function exportTextDocument(options: {
  baseName: string
  content: string
  extension: ExportExtension
}): Promise<string> {
  const { fileName } = await normalizeExportFilename({
    baseName: inferExportBaseName(options.baseName),
    extension: options.extension,
  })

  const path = await save({
    defaultPath: fileName,
    filters: [
      {
        name: options.extension === 'md' ? 'Markdown 文件' : '文本文件',
        extensions: [options.extension],
      },
    ],
  })

  if (!path) {
    throw new Error('已取消导出。')
  }

  await saveTextExport({
    content: options.content,
    path,
  })

  return fileName
}
