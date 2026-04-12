<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import Sortable, { type SortableEvent } from 'sortablejs'

import type { ActiveImageSource } from '../lib/types'

type PickerItem = {
  id: string
  file: File
  previewUrl: string
}

const MAX_UPLOAD_IMAGES = 30

const props = defineProps<{
  activeImages: ActiveImageSource[]
  batchCurrentImageName: string
  batchFolderPath: string
  batchLogs: string[]
  batchProcessing: boolean
  batchProgressLabel: string
  batchProgressPercent: number
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
  'run-folder-batch': []
  'run-pipeline': []
  'set-preview-index': [index: number]
  'update:url-input': [value: string]
}>()

const dragActive = ref(false)
const pickerOpen = ref(false)
const pickerItems = ref<PickerItem[]>([])
const pickerActiveIndex = ref(0)
const pickerDragging = ref(false)
const pickerError = ref('')
const pickerFileInput = ref<HTMLInputElement | null>(null)
const pickerListRef = ref<HTMLElement | null>(null)
let pickerSortable: Sortable | null = null

const lightboxOpen = ref(false)
const lightboxIndex = ref(0)

const firstImageName = computed(() => props.activeImages[0]?.displayName ?? '')
const pickerActiveItem = computed(() => pickerItems.value[pickerActiveIndex.value] ?? null)
const lightboxImage = computed(() => props.activeImages[lightboxIndex.value] ?? null)

function createPickerItem(file: File): PickerItem {
  return {
    id: `picker-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`,
    file,
    previewUrl: URL.createObjectURL(file),
  }
}

function revokePickerItemUrls(items: PickerItem[]) {
  items.forEach((item) => URL.revokeObjectURL(item.previewUrl))
}

function destroyPickerSortable() {
  if (pickerSortable) {
    pickerSortable.destroy()
    pickerSortable = null
  }
}

function resetPicker(items: PickerItem[] = []) {
  destroyPickerSortable()
  revokePickerItemUrls(pickerItems.value)
  pickerItems.value = items
  pickerActiveIndex.value = items.length ? 0 : 0
  pickerError.value = ''
  pickerDragging.value = false
}

function collectFiles(list: FileList | null): File[] {
  if (!list) {
    return []
  }

  return Array.from(list).filter((file) => file.type.startsWith('image/'))
}

function appendPickerFiles(files: File[]) {
  const onlyImages = files.filter((file) => file.type.startsWith('image/'))
  if (!onlyImages.length) {
    pickerError.value = '仅支持图片文件。'
    return
  }

  const remaining = MAX_UPLOAD_IMAGES - pickerItems.value.length
  if (remaining <= 0) {
    pickerError.value = `最多仅支持 ${MAX_UPLOAD_IMAGES} 张图片。`
    return
  }

  const accepted = onlyImages.slice(0, remaining)
  const appended = accepted.map(createPickerItem)
  pickerItems.value = [...pickerItems.value, ...appended]

  if (pickerItems.value.length === appended.length) {
    pickerActiveIndex.value = 0
  }

  if (accepted.length < onlyImages.length) {
    pickerError.value = `最多仅支持 ${MAX_UPLOAD_IMAGES} 张图片，超出部分已忽略。`
  } else {
    pickerError.value = ''
  }
}

function buildPickerItemsFromActiveImages(images: ActiveImageSource[]): PickerItem[] {
  return images.map((image) => {
    const bytes = new Uint8Array(image.imageBytes)
    const file = new File([bytes], image.displayName, {
      type: image.mimeType || 'image/png',
    })
    return createPickerItem(file)
  })
}

function reorderPickerItems(fromIndex: number, toIndex: number) {
  if (
    fromIndex === toIndex ||
    fromIndex < 0 ||
    toIndex < 0 ||
    fromIndex >= pickerItems.value.length ||
    toIndex >= pickerItems.value.length
  ) {
    return
  }

  const next = [...pickerItems.value]
  const [moved] = next.splice(fromIndex, 1)
  next.splice(toIndex, 0, moved)
  pickerItems.value = next

  if (pickerActiveIndex.value === fromIndex) {
    pickerActiveIndex.value = toIndex
  } else if (fromIndex < pickerActiveIndex.value && toIndex >= pickerActiveIndex.value) {
    pickerActiveIndex.value -= 1
  } else if (fromIndex > pickerActiveIndex.value && toIndex <= pickerActiveIndex.value) {
    pickerActiveIndex.value += 1
  }
}

function onPickerSortEnd(event: SortableEvent) {
  pickerDragging.value = false

  const fromIndex = event.oldIndex
  const toIndex = event.newIndex

  if (fromIndex == null || toIndex == null || fromIndex === toIndex) {
    return
  }

  reorderPickerItems(fromIndex, toIndex)
}

function initPickerSortable() {
  destroyPickerSortable()

  if (!pickerOpen.value || !pickerListRef.value || pickerItems.value.length < 2) {
    return
  }

  pickerSortable = Sortable.create(pickerListRef.value, {
    animation: 150,
    draggable: '.picker-item',
    handle: '.drag-handle',
    forceFallback: true,
    fallbackTolerance: 3,
    ghostClass: 'is-sort-ghost',
    chosenClass: 'is-sort-chosen',
    dragClass: 'is-sort-drag',
    onStart: () => {
      pickerDragging.value = true
    },
    onEnd: onPickerSortEnd,
  })
}

function openPicker() {
  emit('lock-url-import')
  const initialItems = buildPickerItemsFromActiveImages(props.activeImages)
  resetPicker(initialItems)
  pickerOpen.value = true
  void nextTick(() => initPickerSortable())
}

function closePicker() {
  pickerOpen.value = false
  resetPicker([])
  if (pickerFileInput.value) {
    pickerFileInput.value.value = ''
  }
}

function openPickerFileInput() {
  pickerFileInput.value?.click()
}

function onPickerFileChange(event: Event) {
  const input = event.target as HTMLInputElement
  appendPickerFiles(collectFiles(input.files))
  input.value = ''
}

function onPickerDragOver(event: DragEvent) {
  event.preventDefault()
}

function onPickerDropZone(event: DragEvent) {
  event.preventDefault()
  const listFiles = collectFiles(event.dataTransfer?.files ?? null)
  if (listFiles.length) {
    appendPickerFiles(listFiles)
  }
}

function removePickerItem(index: number) {
  if (index < 0 || index >= pickerItems.value.length) {
    return
  }

  const next = [...pickerItems.value]
  const [removed] = next.splice(index, 1)
  URL.revokeObjectURL(removed.previewUrl)
  pickerItems.value = next

  if (!next.length) {
    pickerActiveIndex.value = 0
    return
  }

  if (pickerActiveIndex.value >= next.length) {
    pickerActiveIndex.value = next.length - 1
  }
}

function selectPickerItem(index: number) {
  if (pickerDragging.value) {
    return
  }
  pickerActiveIndex.value = index
}

function confirmPickerSelection() {
  if (!pickerItems.value.length) {
    pickerError.value = '请先上传至少 1 张图片。'
    return
  }

  emit('import-files', pickerItems.value.map((item) => item.file))
  emit('set-preview-index', 0)
  closePicker()
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

function openLightbox(index: number) {
  if (!props.activeImages.length) {
    return
  }

  lightboxIndex.value = Math.min(Math.max(index, 0), props.activeImages.length - 1)
  lightboxOpen.value = true
  emit('set-preview-index', lightboxIndex.value)
}

function closeLightbox() {
  lightboxOpen.value = false
}

function lightboxPrev() {
  if (!props.activeImages.length) {
    return
  }

  lightboxIndex.value = (lightboxIndex.value - 1 + props.activeImages.length) % props.activeImages.length
  emit('set-preview-index', lightboxIndex.value)
}

function lightboxNext() {
  if (!props.activeImages.length) {
    return
  }

  lightboxIndex.value = (lightboxIndex.value + 1) % props.activeImages.length
  emit('set-preview-index', lightboxIndex.value)
}

watch(
  () => props.activeImages.length,
  (count) => {
    if (!count) {
      lightboxOpen.value = false
      lightboxIndex.value = 0
      return
    }

    if (lightboxIndex.value > count - 1) {
      lightboxIndex.value = count - 1
    }
  },
)

watch(
  () => [pickerOpen.value, pickerItems.value.length] as const,
  async ([open]) => {
    if (!open) {
      destroyPickerSortable()
      return
    }

    await nextTick()
    initPickerSortable()
  },
)

onBeforeUnmount(() => {
  destroyPickerSortable()
  resetPicker([])
})
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
      <button class="primary-button" type="button" :disabled="busy" @click="openPicker">
        选择图片（弹窗管理）
      </button>
      <button class="secondary-button" type="button" :disabled="busy" @click="emit('run-folder-batch')">
        选择文件夹批处理
      </button>
      <button v-if="showUrlImport" class="secondary-button" type="button" :disabled="busy" @click="emit('open-web-portal')">
        打开网页 OCR
      </button>
    </div>

    <div class="folder-batch-card">
      <div class="folder-batch-header">
        <p class="summary-label">文件夹批处理</p>
        <span class="meta-chip">{{ props.batchProgressPercent }}%</span>
      </div>
      <p class="summary-muted">
        {{ props.batchFolderPath ? `已选择：${props.batchFolderPath}` : '尚未选择文件夹。' }}
      </p>
      <div class="folder-progress-track" role="progressbar" :aria-valuenow="props.batchProgressPercent" aria-valuemin="0" aria-valuemax="100">
        <div class="folder-progress-fill" :style="{ width: `${props.batchProgressPercent}%` }" />
      </div>
      <p class="summary-muted">
        {{ props.batchProgressLabel ? `进度：${props.batchProgressLabel}` : '进度：等待执行' }}
        <template v-if="props.batchCurrentImageName">，当前：{{ props.batchCurrentImageName }}</template>
      </p>
      <div class="folder-batch-log">
        <p v-if="!props.batchLogs.length" class="summary-muted">日志将在执行时实时显示。</p>
        <ul v-else class="folder-batch-log-list">
          <li v-for="(line, index) in props.batchLogs" :key="`${index}-${line}`">
            {{ line }}
          </li>
        </ul>
      </div>
      <p class="flow-tip">执行后会在每个图片目录生成 1 个 Markdown，并在根目录额外生成 1 个总 Markdown（提示词仅展示一次）。</p>
      <p v-if="props.batchProcessing" class="summary-muted">正在批处理，请稍候...</p>
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

      <ul v-if="activeImages.length" class="selected-image-list">
        <li v-for="(image, index) in activeImages" :key="image.id">
          <button class="selected-image-item" type="button" @click="openLightbox(index)">
            <span class="selected-image-index">{{ index + 1 }}</span>
            <span class="selected-image-name">{{ image.displayName }}</span>
            <span class="selected-image-action">放大预览</span>
          </button>
        </li>
      </ul>

      <button class="primary-button flow-run-button" type="button" :disabled="busy || !activeImages.length" @click="emit('run-pipeline')">
        {{ pipelineBusy ? '正在执行一键流程...' : '开始一键处理（全图 OCR + AI + 汇总）' }}
      </button>
      <p class="flow-tip">OCR 将按上传顺序输出全部图片结果，AI 对全部结果统一结构化分析。</p>
    </div>
  </section>

  <div v-if="pickerOpen" class="modal-backdrop" @click.self="closePicker">
    <section class="modal-card picker-modal-card">
      <div class="panel-heading">
        <div>
          <p class="panel-kicker">图片选择弹窗</p>
          <h2>上传、删除、排序与预览</h2>
        </div>
        <button class="ghost-button" type="button" :disabled="busy" @click="closePicker">关闭</button>
      </div>

      <div class="picker-toolbar">
        <button class="primary-button" type="button" :disabled="busy || pickerItems.length >= MAX_UPLOAD_IMAGES" @click="openPickerFileInput">
          添加图片
        </button>
        <span class="field-hint">已选 {{ pickerItems.length }} / {{ MAX_UPLOAD_IMAGES }} 张</span>
      </div>

      <div class="picker-grid">
        <div class="picker-panel">
          <div class="picker-drop-zone" @dragover="onPickerDragOver" @drop="onPickerDropZone">
            <p class="drop-title">拖拽图片到此区域添加</p>
            <p class="drop-desc">可继续添加，超出 30 张会自动忽略。</p>
          </div>

          <ul ref="pickerListRef" class="picker-list">
            <li
              v-for="(item, index) in pickerItems"
              :key="item.id"
              class="picker-item"
              :class="{ 'is-active': index === pickerActiveIndex }"
              @click="selectPickerItem(index)"
            >
              <span class="drag-handle" title="拖拽排序" aria-hidden="true">⋮⋮</span>
              <img :src="item.previewUrl" :alt="item.file.name" class="picker-thumb" draggable="false" />
              <div class="picker-item-meta">
                <p class="picker-item-name">{{ index + 1 }}. {{ item.file.name }}</p>
                <p class="picker-item-size">{{ Math.max(1, Math.round(item.file.size / 1024)) }} KB</p>
              </div>
              <button
                class="ghost-button picker-delete-button"
                type="button"
                :disabled="busy"
                draggable="false"
                @click.stop="removePickerItem(index)"
              >
                删除
              </button>
            </li>
          </ul>
        </div>

        <div class="picker-panel picker-preview-panel">
          <div v-if="pickerActiveItem" class="picker-preview-frame">
            <img :src="pickerActiveItem.previewUrl" :alt="pickerActiveItem.file.name" class="picker-preview-image" />
          </div>
          <div v-else class="empty-state">
            <p>请先添加图片。</p>
          </div>
          <p v-if="pickerActiveItem" class="field-hint">当前预览：{{ pickerActiveItem.file.name }}</p>
        </div>
      </div>

      <p v-if="pickerError" class="status-error">{{ pickerError }}</p>

      <div class="button-row">
        <button class="secondary-button" type="button" :disabled="busy" @click="closePicker">取消</button>
        <button class="primary-button" type="button" :disabled="busy || !pickerItems.length" @click="confirmPickerSelection">
          确定并应用顺序
        </button>
      </div>

      <input
        ref="pickerFileInput"
        accept="image/*"
        class="visually-hidden"
        multiple
        type="file"
        @change="onPickerFileChange"
      />
    </section>
  </div>

  <div v-if="lightboxOpen && lightboxImage" class="modal-backdrop" @click.self="closeLightbox">
    <section class="modal-card lightbox-modal-card">
      <div class="panel-heading">
        <div>
          <p class="panel-kicker">已选图片预览</p>
          <h2>{{ lightboxImage.displayName }}</h2>
        </div>
        <span class="meta-chip">{{ lightboxIndex + 1 }}/{{ activeImages.length }}</span>
      </div>

      <div class="lightbox-frame">
        <img :src="lightboxImage.previewUrl" :alt="lightboxImage.displayName" class="lightbox-image" />
      </div>

      <div class="button-row">
        <button class="secondary-button" type="button" :disabled="activeImages.length <= 1" @click="lightboxPrev">上一张</button>
        <button class="secondary-button" type="button" :disabled="activeImages.length <= 1" @click="lightboxNext">下一张</button>
        <button class="ghost-button" type="button" @click="closeLightbox">关闭</button>
      </div>
    </section>
  </div>
</template>
