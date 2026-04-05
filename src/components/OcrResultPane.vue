<script setup lang="ts">
import type { OcrResult } from '../lib/types'

defineProps<{
  canExport: boolean
  ocrResult: OcrResult | null
}>()

const emit = defineEmits<{
  export: []
}>()
</script>

<template>
  <section class="panel result-panel">
    <div class="panel-heading">
      <div>
        <p class="panel-kicker">流程输出</p>
        <h2>OCR 识别结果</h2>
      </div>
      <span v-if="ocrResult" class="result-meta">耗时 {{ ocrResult.durationMs }} ms</span>
    </div>

    <textarea
      class="result-textarea"
      readonly
      :value="ocrResult?.text ?? ''"
      placeholder="一键流程执行后，OCR 文本会显示在这里。"
    />

    <div class="button-row">
      <button class="secondary-button" type="button" :disabled="!canExport" @click="emit('export')">
        导出 TXT
      </button>
    </div>
  </section>
</template>
