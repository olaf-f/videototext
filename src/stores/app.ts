import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { openUrl } from '@tauri-apps/plugin-opener'

import {
  importImageFromUrl,
  loadSettings,
  runOcr as invokeRunOcr,
  saveDeepseekApiKey as invokeSaveDeepseekApiKey,
  saveSettings as invokeSaveSettings,
  structureTextWithDeepseek,
  validateOcrAssets,
} from '../lib/api'
import { copyHtmlAndText, copyPlainText } from '../lib/clipboard'
import {
  appendAccumulatorSection,
  exportTextDocument,
  inferExportBaseName,
  type ExportExtension,
} from '../lib/export'
import { renderMarkdown } from '../lib/markdown'
import type {
  ActiveImageSource,
  AiResult,
  AppSettings,
  BusyState,
  OcrResult,
  OcrResultItem,
  SaveSettingsPayload,
} from '../lib/types'

const MAX_UPLOAD_IMAGES = 30

const defaultSettings: AppSettings = {
  deepseekApiKeySaved: false,
  defaultPrompt: '请提取图像中的关键信息，并输出简洁的 Markdown 结构化摘要。',
  webPortalUrl: 'https://example.com/web-ocr',
}

function createImageId() {
  return `image-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`
}

function toErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

async function fileToBytes(file: Blob): Promise<number[]> {
  return Array.from(new Uint8Array(await file.arrayBuffer()))
}

function buildMergedOcrText(items: OcrResultItem[]): string {
  return items
    .map((item) => {
      const content = item.text.trim() || '（未识别到有效文本）'
      return `## 图片 ${item.order}/${items.length}：${item.displayName}\n\n${content}`
    })
    .join('\n\n')
}

export const useAppStore = defineStore('app', () => {
  const accumulatorMarkdown = ref('')
  const activeImages = ref<ActiveImageSource[]>([])
  const currentImageIndex = ref(0)
  const aiResult = ref<AiResult | null>(null)
  const busyState = ref<BusyState>(null)
  const errorMessage = ref('')
  const initialized = ref(false)
  const isSettingsOpen = ref(false)
  const ocrResult = ref<OcrResult | null>(null)
  const promptInput = ref(defaultSettings.defaultPrompt)
  const settings = ref<AppSettings>(defaultSettings)
  const startupError = ref('')
  const statusMessage = ref('请先上传图片，然后点击“一键处理”。')
  const urlInput = ref('')
  const urlImportLocked = ref(false)
  const operationToken = ref(0)

  const activeImage = computed(() => activeImages.value[currentImageIndex.value] ?? null)

  const busyLabel = computed(() => {
    switch (busyState.value) {
      case 'import':
        return '正在导入图片...'
      case 'ocr':
        return '正在执行 OCR 识别...'
      case 'ai':
        return '正在进行 AI 结构化...'
      case 'pipeline':
        return '正在执行一键流程...'
      case 'settings':
        return '正在保存设置...'
      case 'export':
        return '正在准备导出...'
      default:
        return ''
    }
  })

  function resetResults() {
    ocrResult.value = null
    aiResult.value = null
  }

  function revokePreviewUrls(images: ActiveImageSource[]) {
    images.forEach((image) => {
      if (image.previewUrl.startsWith('blob:')) {
        URL.revokeObjectURL(image.previewUrl)
      }
    })
  }

  function setActiveImages(nextImages: ActiveImageSource[]) {
    revokePreviewUrls(activeImages.value)
    activeImages.value = nextImages
    currentImageIndex.value = 0
    resetResults()
  }

  function setCurrentImageIndex(index: number) {
    if (!activeImages.value.length) {
      currentImageIndex.value = 0
      return
    }

    const maxIndex = activeImages.value.length - 1
    currentImageIndex.value = Math.min(Math.max(index, 0), maxIndex)
  }

  function reorderActiveImages(fromIndex: number, toIndex: number) {
    const total = activeImages.value.length
    if (total < 2) {
      return
    }

    if (
      fromIndex < 0 ||
      toIndex < 0 ||
      fromIndex >= total ||
      toIndex >= total ||
      fromIndex === toIndex
    ) {
      return
    }

    const currentId = activeImage.value?.id
    const next = [...activeImages.value]
    const [moved] = next.splice(fromIndex, 1)
    next.splice(toIndex, 0, moved)
    activeImages.value = next

    const nextIndex = currentId ? next.findIndex((image) => image.id === currentId) : 0
    currentImageIndex.value = nextIndex >= 0 ? nextIndex : 0
    resetResults()
    statusMessage.value = '图片顺序已更新，请重新执行一键处理。'
  }

  function setUrlInput(value: string) {
    urlInput.value = value
  }

  function setPromptInput(value: string) {
    promptInput.value = value
  }

  function setAccumulatorMarkdown(value: string) {
    accumulatorMarkdown.value = value
  }

  function lockUrlImport() {
    urlImportLocked.value = true
  }

  function openSettings() {
    isSettingsOpen.value = true
  }

  function closeSettings() {
    isSettingsOpen.value = false
  }

  async function withBusy<T>(state: BusyState, work: () => Promise<T>): Promise<T> {
    if (busyState.value) {
      throw new Error('当前已有任务执行中，请稍后重试。')
    }

    const token = ++operationToken.value
    busyState.value = state
    errorMessage.value = ''
    try {
      return await work()
    } catch (error) {
      const message = toErrorMessage(error)
      errorMessage.value = message
      statusMessage.value = message
      throw error
    } finally {
      if (operationToken.value === token) {
        busyState.value = null
      }
    }
  }

  async function performOcr(image: ActiveImageSource, order: number): Promise<OcrResultItem> {
    const startedAt = performance.now()
    const response = await invokeRunOcr({ imageBytes: image.imageBytes })

    return {
      order,
      sourceId: image.id,
      displayName: image.displayName,
      text: response.text,
      durationMs: Math.round(performance.now() - startedAt),
    }
  }

  async function performAi(ocrText: string, sourceIds: string[]): Promise<AiResult> {
    const startedAt = performance.now()
    const response = await structureTextWithDeepseek({
      ocrText,
      prompt: promptInput.value.trim() || settings.value.defaultPrompt,
    })

    return {
      markdown: response.markdown,
      durationMs: Math.round(performance.now() - startedAt),
      sourceIds,
    }
  }

  async function runOcrOnAllImages(images: ActiveImageSource[]): Promise<OcrResult> {
    const items: OcrResultItem[] = []

    for (let i = 0; i < images.length; i += 1) {
      const image = images[i]
      statusMessage.value = `OCR ${i + 1}/${images.length}：${image.displayName}`
      const item = await performOcr(image, i + 1)
      items.push(item)
    }

    return {
      items,
      text: buildMergedOcrText(items),
      durationMs: items.reduce((sum, item) => sum + item.durationMs, 0),
    }
  }

  async function initialize() {
    if (initialized.value) {
      return
    }

    try {
      const loadedSettings = await loadSettings()
      settings.value = loadedSettings
      promptInput.value = loadedSettings.defaultPrompt || defaultSettings.defaultPrompt
      await validateOcrAssets()
      startupError.value = ''
      statusMessage.value = '设置已加载，系统就绪。'
    } catch (error) {
      const message = toErrorMessage(error)
      errorMessage.value = message
      startupError.value = message
      statusMessage.value = 'OCR 资源异常，请先修复后再执行离线识别。'
    } finally {
      initialized.value = true
    }
  }

  async function importFromFiles(files: File[]) {
    await withBusy('import', async () => {
      const onlyImages = files.filter((file) => file.type.startsWith('image/'))
      if (!onlyImages.length) {
        throw new Error('未检测到可用图片文件。')
      }

      if (onlyImages.length > MAX_UPLOAD_IMAGES) {
        throw new Error(`一次最多上传 ${MAX_UPLOAD_IMAGES} 张图片。`)
      }

      const images = await Promise.all(
        onlyImages.map(async (file) => ({
          id: createImageId(),
          sourceType: 'file' as const,
          displayName: file.name,
          mimeType: file.type || 'application/octet-stream',
          previewUrl: URL.createObjectURL(file),
          byteLength: file.size,
          imageBytes: await fileToBytes(file),
        })),
      )

      lockUrlImport()
      setActiveImages(images)
      statusMessage.value = `已选择 ${images.length} 张图片，可执行一键处理。`
    })
  }

  async function importFromUrl(rawUrl: string) {
    const nextUrl = rawUrl.trim()
    await withBusy('import', async () => {
      if (!nextUrl) {
        throw new Error('请先输入图片直链 URL。')
      }

      const imported = await importImageFromUrl(nextUrl)
      const blob = new Blob([new Uint8Array(imported.imageBytes)], {
        type: imported.contentType || 'image/png',
      })
      const previewUrl = URL.createObjectURL(blob)
      const image: ActiveImageSource = {
        id: createImageId(),
        sourceType: 'url',
        displayName: imported.displayName || nextUrl.split('/').pop() || 'URL 导入图片',
        mimeType: imported.contentType || blob.type || 'image/png',
        previewUrl,
        byteLength: imported.imageBytes.length,
        imageBytes: imported.imageBytes,
      }
      setActiveImages([image])
      urlInput.value = imported.url
      statusMessage.value = '已从 URL 导入图片。'
    })
  }

  async function runOcr() {
    if (!activeImages.value.length) {
      return
    }

    await withBusy('ocr', async () => {
      ocrResult.value = await runOcrOnAllImages(activeImages.value)
      statusMessage.value = `OCR 完成，共识别 ${ocrResult.value.items.length} 张图片。`
    })
  }

  async function runAi() {
    const ocr = ocrResult.value
    if (!ocr?.text.trim()) {
      return
    }

    await withBusy('ai', async () => {
      aiResult.value = await performAi(
        ocr.text,
        ocr.items.map((item) => item.sourceId),
      )
      statusMessage.value = 'AI 结构化完成。'
    })
  }

  async function runPipeline() {
    const images = activeImages.value
    if (!images.length) {
      throw new Error('请先上传图片，再执行一键流程。')
    }

    await withBusy('pipeline', async () => {
      const ocr = await runOcrOnAllImages(images)
      ocrResult.value = ocr

      statusMessage.value = '流程 2/3：AI 结构化中...'
      let ai: AiResult | null = null
      if (ocr.text.trim()) {
        ai = await performAi(
          ocr.text,
          ocr.items.map((item) => item.sourceId),
        )
        aiResult.value = ai
      } else {
        aiResult.value = null
      }

      statusMessage.value = '流程 3/3：写入汇总中...'
      if (ai?.markdown.trim()) {
        accumulatorMarkdown.value = appendAccumulatorSection(
          accumulatorMarkdown.value,
          'AI 结构化结果（全图分析）',
          ai.markdown,
        )
      }

      statusMessage.value = `一键流程完成：共 ${images.length} 张图片。`
    })
  }

  async function copyOcrResult() {
    if (!ocrResult.value?.text.trim()) {
      return
    }

    await withBusy('export', async () => {
      await copyPlainText(ocrResult.value!.text)
      statusMessage.value = 'OCR 结果已复制到剪贴板。'
    })
  }

  async function exportDocument(content: string, extension: ExportExtension, baseName: string) {
    await withBusy('export', async () => {
      const fileName = await exportTextDocument({ baseName, content, extension })
      statusMessage.value = `导出文件已生成：${fileName}`
    })
  }

  async function exportOcrResult() {
    if (!ocrResult.value) {
      return
    }

    await exportDocument(
      ocrResult.value.text,
      'txt',
      inferExportBaseName(activeImage.value?.displayName ?? 'ocr-batch-result'),
    )
  }

  async function exportAiResult() {
    if (!aiResult.value) {
      return
    }

    await exportDocument(
      aiResult.value.markdown,
      'md',
      inferExportBaseName(activeImage.value?.displayName ?? 'ai-batch-result'),
    )
  }

  async function exportAccumulator(extension: ExportExtension) {
    if (!accumulatorMarkdown.value.trim()) {
      return
    }

    const content =
      extension === 'txt'
        ? accumulatorMarkdown.value.replace(/^##\s+/gm, '').trim()
        : accumulatorMarkdown.value

    await exportDocument(content, extension, 'smartocr-pro-accumulator')
  }

  async function copyAiResult() {
    if (!aiResult.value?.markdown.trim()) {
      return
    }

    await withBusy('export', async () => {
      const markdown = aiResult.value!.markdown
      const html = renderMarkdown(markdown)
      await copyHtmlAndText(html, markdown)
      statusMessage.value = 'AI 结果已复制到剪贴板。'
    })
  }

  function clearAccumulator() {
    accumulatorMarkdown.value = ''
    statusMessage.value = '汇总区已清空。'
  }

  async function saveSettings(nextSettings: SaveSettingsPayload) {
    await withBusy('settings', async () => {
      const previousDefaultPrompt = settings.value.defaultPrompt
      const nextApiKey = nextSettings.deepseekApiKey.trim()
      if (nextApiKey) {
        await invokeSaveDeepseekApiKey(nextApiKey)
        settings.value = {
          ...settings.value,
          deepseekApiKeySaved: true,
        }
      }

      const payload: AppSettings = {
        deepseekApiKeySaved: settings.value.deepseekApiKeySaved || Boolean(nextApiKey),
        defaultPrompt: nextSettings.defaultPrompt,
        webPortalUrl: nextSettings.webPortalUrl,
      }
      await invokeSaveSettings(payload)
      const loadedSettings = await loadSettings()
      settings.value = {
        ...loadedSettings,
        deepseekApiKeySaved: loadedSettings.deepseekApiKeySaved || Boolean(nextApiKey),
      }
      if (!promptInput.value.trim() || promptInput.value === previousDefaultPrompt) {
        promptInput.value = settings.value.defaultPrompt
      }
      isSettingsOpen.value = false
      statusMessage.value = nextApiKey ? '设置已保存，DeepSeek API Key 已立即生效。' : '设置已保存。'
    })
  }

  async function openWebPortal() {
    try {
      await openUrl(settings.value.webPortalUrl)
      statusMessage.value = '已打开网页 OCR 门户。'
    } catch (error) {
      errorMessage.value = toErrorMessage(error)
      statusMessage.value = errorMessage.value
    }
  }

  function dispose() {
    revokePreviewUrls(activeImages.value)
  }

  return {
    accumulatorMarkdown,
    activeImage,
    activeImages,
    aiResult,
    busyLabel,
    busyState,
    clearAccumulator,
    closeSettings,
    copyAiResult,
    copyOcrResult,
    currentImageIndex,
    dispose,
    errorMessage,
    exportAccumulator,
    exportAiResult,
    exportOcrResult,
    importFromFiles,
    importFromUrl,
    initialize,
    isSettingsOpen,
    lockUrlImport,
    ocrResult,
    openSettings,
    openWebPortal,
    promptInput,
    runAi,
    runOcr,
    runPipeline,
    reorderActiveImages,
    saveSettings,
    setAccumulatorMarkdown,
    setCurrentImageIndex,
    setPromptInput,
    setUrlInput,
    settings,
    startupError,
    statusMessage,
    urlImportLocked,
    urlInput,
  }
})
