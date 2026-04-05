<script setup lang="ts">
import { computed, ref } from 'vue'

import type { ActiveImageSource } from '../lib/types'

const props = defineProps<{
  activeImages: ActiveImageSource[]
  busy: boolean
  pipelineBusy: boolean
  showUrlImport: boolean
  urlInput: string
}>()

const emit = defineEmits<{
  'import-files': [files: File[]]
  'import-url': [url: string]
  'lock-url-import': []
  'open-settings': []
  'open-web-portal': []
  'run-pipeline': []
  'update:url-input': [value: string]
}>()

const fileInput = ref<HTMLInputElement | null>(null)
const dragActive = ref(false)

const firstImageName = computed(() => props.activeImages[0]?.displayName ?? '')

function openFilePicker() {
  emit('lock-url-import')
  fileInput.value?.click()
}

function collectFiles(list: FileList | null): File[] {
  if (!list) {
    return []
  }

  return Array.from(list).filter((file) => file.type.startsWith('image/'))
}

function onFileChange(event: Event) {
  const input = event.target as HTMLInputElement
  const files = collectFiles(input.files)
  if (files.length) {
    emit('import-files', files)
  }

  input.value = ''
}

function onDragOver(event: DragEvent) {
  event.preventDefault()
  dragActive.value = true
}

function onDragLeave(event: DragEvent) {
  event.preventDefault()
  dragActive.value = false
}

function onDrop(event: DragEvent) {
  event.preventDefault()
  dragActive.value = false
  const files = collectFiles(event.dataTransfer?.files ?? null)
  if (!files.length) {
    return
  }

  emit('lock-url-import')
  emit('import-files', files)
}

function submitUrlImport() {
  emit('import-url', props.urlInput)
}
</script>

<template>
  <section class="panel">
    <div class="panel-heading panel-heading-compact">
      <div>
        <p class="panel-kicker">一体化流程</p>
        <h2>导入与处理</h2>
      </div>
      <button class="ghost-button" type="button" :disabled="busy" @click="emit('open-settings')">设置</button>
    </div>

    <div class="button-row">
      <button class="primary-button" type="button" :disabled="busy" @click="openFilePicker">
        选择图片（最多30张）
      </button>
      <button v-if="showUrlImport" class="secondary-button" type="button" :disabled="busy" @click="emit('open-web-portal')">
        打开网页 OCR
      </button>
    </div>

    <div
      class="upload-drop-zone"
      :class="{ 'is-drag-active': dragActive }"
      @dragover="onDragOver"
      @dragleave="onDragLeave"
      @drop="onDrop"
    >
      <p class="drop-title">拖拽图片到此上传区域</p>
      <p class="drop-desc">支持 JPG / PNG / WebP / BMP / TIFF，单次最多 30 张</p>
    </div>

    <label v-if="showUrlImport" class="field">
      <span>图片直链 URL</span>
      <div class="inline-form">
        <input
          :value="urlInput"
          class="text-input"
          placeholder="https://example.com/image.png"
          type="url"
          @input="emit('update:url-input', ($event.target as HTMLInputElement).value)"
          @keydown.enter.prevent="submitUrlImport"
        />
        <button class="secondary-button" type="button" :disabled="busy" @click="submitUrlImport">导入 URL</button>
      </div>
    </label>

    <div class="flow-card">
      <p class="summary-label">已选图片</p>
      <p v-if="activeImages.length" class="summary-value">
        共 {{ activeImages.length }} 张，当前预览：{{ firstImageName }}
      </p>
      <p v-else class="summary-muted">尚未选择图片。</p>

      <button class="primary-button flow-run-button" type="button" :disabled="busy || !activeImages.length" @click="emit('run-pipeline')">
        {{ pipelineBusy ? '正在执行一键流程...' : '开始一键处理（全图 OCR + AI + 汇总）' }}
      </button>
      <p class="flow-tip">OCR 将按上传顺序输出全部图片结果，AI 对全部结果统一结构化分析。</p>
    </div>

    <input
      ref="fileInput"
      accept="image/*"
      class="visually-hidden"
      multiple
      type="file"
      @change="onFileChange"
    />
  </section>
</template>
