<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from 'vue'
import { storeToRefs } from 'pinia'

import AccumulatorPane from './components/AccumulatorPane.vue'
import ImageInputPanel from './components/ImageInputPanel.vue'
import ImagePreviewPane from './components/ImagePreviewPane.vue'
import ProcessOutputTabs from './components/ProcessOutputTabs.vue'
import SettingsDialog from './components/SettingsDialog.vue'
import StatusBar from './components/StatusBar.vue'
import { useAppStore } from './stores/app'

const appStore = useAppStore()

const {
  accumulatorMarkdown,
  activeImage,
  activeImages,
  aiResult,
  busyLabel,
  busyState,
  currentImageIndex,
  errorMessage,
  folderBatchCurrentImageName,
  folderBatchLogs,
  folderBatchPath,
  folderBatchProgressLabel,
  folderBatchProgressPercent,
  isSettingsOpen,
  ocrResult,
  promptInput,
  settings,
  startupError,
  statusMessage,
  urlImportLocked,
  urlInput,
} = storeToRefs(appStore)

const isAnyBusy = computed(() => Boolean(busyState.value))
const isPipelineBusy = computed(() => busyState.value === 'pipeline')
const isFolderBatchBusy = computed(() => busyState.value === 'folder-batch')
const isSettingsBusy = computed(() => busyState.value === 'settings')
const showUrlImport = computed(
  () =>
    !urlImportLocked.value &&
    !isFolderBatchBusy.value &&
    !folderBatchPath.value.trim(),
)

function onReorderImages(payload: { fromIndex: number; toIndex: number }) {
  appStore.reorderActiveImages(payload.fromIndex, payload.toIndex)
}

onMounted(() => {
  void appStore.initialize()
})

onBeforeUnmount(() => {
  appStore.dispose()
})
</script>

<template>
  <main class="app-shell">
    <header class="app-header">
      <div>
        <p class="eyebrow">离线 OCR + AI 一体化工作台</p>
        <h1>SmartOCR Pro 中文版</h1>
      </div>
      <p class="lead">
        支持拖拽或弹窗一次上传最多 30 张图片，OCR 按上传顺序输出全部结果，AI 对全量结果统一结构化分析。
      </p>
    </header>

    <StatusBar
      :active-image="activeImage"
      :busy-label="busyLabel"
      :error-message="errorMessage"
      :image-count="activeImages.length"
      :settings="settings"
      :status-message="statusMessage"
    />

    <p v-if="startupError" class="startup-error">
      {{ startupError }}
    </p>

    <section class="workspace-grid four-panel-grid">
      <ImageInputPanel
        :active-images="activeImages"
        :busy="isAnyBusy"
        :batch-current-image-name="folderBatchCurrentImageName"
        :batch-folder-path="folderBatchPath"
        :batch-logs="folderBatchLogs"
        :batch-processing="isFolderBatchBusy"
        :batch-progress-label="folderBatchProgressLabel"
        :batch-progress-percent="folderBatchProgressPercent"
        :pipeline-busy="isPipelineBusy"
        :show-url-import="showUrlImport"
        :url-input="urlInput"
        @import-files="appStore.importFromFiles"
        @import-url="appStore.importFromUrl"
        @lock-url-import="appStore.lockUrlImport()"
        @open-settings="appStore.openSettings()"
        @open-web-portal="appStore.openWebPortal()"
        @run-folder-batch="appStore.runFolderBatchFromDialog()"
        @run-pipeline="appStore.runPipeline()"
        @set-preview-index="appStore.setCurrentImageIndex"
        @update:url-input="appStore.setUrlInput"
      />

      <ImagePreviewPane
        :active-image="activeImage"
        :active-images="activeImages"
        :image-count="activeImages.length"
        :image-index="currentImageIndex"
        @reorder-images="onReorderImages"
        @set-index="appStore.setCurrentImageIndex"
      />

      <ProcessOutputTabs
        :ai-result="aiResult"
        :busy="isAnyBusy"
        :can-copy="Boolean(aiResult)"
        :can-copy-ocr="Boolean(ocrResult?.text.trim())"
        :can-export-ai="Boolean(aiResult)"
        :can-export-ocr="Boolean(ocrResult)"
        :ocr-result="ocrResult"
        :prompt="promptInput"
        @copy-ai="appStore.copyAiResult()"
        @copy-ocr="appStore.copyOcrResult()"
        @export-ai="appStore.exportAiResult()"
        @export-ocr="appStore.exportOcrResult()"
        @update:prompt="appStore.setPromptInput"
      />

      <AccumulatorPane
        :busy="isAnyBusy"
        :can-export="Boolean(accumulatorMarkdown.trim())"
        :model-value="accumulatorMarkdown"
        @clear="appStore.clearAccumulator()"
        @export-md="appStore.exportAccumulator('md')"
        @export-txt="appStore.exportAccumulator('txt')"
        @update:model-value="appStore.setAccumulatorMarkdown"
      />
    </section>

    <SettingsDialog
      :busy="isSettingsBusy"
      :open="isSettingsOpen"
      :settings="settings"
      @close="appStore.closeSettings()"
      @save="appStore.saveSettings"
    />
  </main>
</template>
