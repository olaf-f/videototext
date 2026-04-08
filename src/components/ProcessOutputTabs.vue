<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import { renderMarkdown } from '../lib/markdown'
import type { AiResult, OcrResult } from '../lib/types'

const props = defineProps<{
  aiResult: AiResult | null
  busy: boolean
  canCopy: boolean
  canCopyOcr: boolean
  canExportAi: boolean
  canExportOcr: boolean
  ocrResult: OcrResult | null
  prompt: string
}>()

const emit = defineEmits<{
  'copy-ai': []
  'copy-ocr': []
  'export-ai': []
  'export-ocr': []
  'update:prompt': [value: string]
}>()

const activeTab = ref<'ocr' | 'ai'>('ocr')
const renderedHtml = computed(() => renderMarkdown(props.aiResult?.markdown ?? ''))

const ocrCharCount = computed(() => props.ocrResult?.text.trim().length ?? 0)
const aiCharCount = computed(() => props.aiResult?.markdown.trim().length ?? 0)
const ocrImageCount = computed(() => props.ocrResult?.items.length ?? 0)

const activeDuration = computed(() => {
  if (activeTab.value === 'ocr') {
    return props.ocrResult?.durationMs ?? null
  }
  return props.aiResult?.durationMs ?? null
})

watch(
  () => props.aiResult?.markdown,
  (value) => {
    if (value?.trim()) {
      activeTab.value = 'ai'
    }
  },
)
</script>

<template>
  <section class="panel output-panel">
    <div class="panel-heading panel-heading-compact">
      <div>
        <p class="panel-kicker">流程输出</p>
        <h2>识别与结构化结果</h2>
      </div>
      <span v-if="activeDuration" class="result-meta">耗时 {{ activeDuration }} ms</span>
    </div>

    <div class="meta-row">
      <span class="meta-chip">OCR {{ ocrImageCount }} 张 / {{ ocrCharCount }} 字</span>
      <span class="meta-chip">AI {{ aiCharCount }} 字</span>
    </div>

    <div class="tab-switch">
      <button
        class="tab-button"
        :class="{ 'is-active': activeTab === 'ocr' }"
        type="button"
        @click="activeTab = 'ocr'"
      >
        OCR 识别结果
      </button>
      <button
        class="tab-button"
        :class="{ 'is-active': activeTab === 'ai' }"
        type="button"
        @click="activeTab = 'ai'"
      >
        AI 结构化结果
      </button>
    </div>

    <div v-if="activeTab === 'ocr'" class="tab-content tab-content-ocr">
      <div class="result-surface plain-surface">
        {{ ocrResult?.text || '执行一键流程后，这里会按上传顺序展示全部图片 OCR 结果。' }}
      </div>

      <div class="button-row">
        <button class="secondary-button" type="button" :disabled="busy || !canCopyOcr" @click="emit('copy-ocr')">
          复制文本
        </button>
        <button class="secondary-button" type="button" :disabled="busy || !canExportOcr" @click="emit('export-ocr')">
          导出 TXT
        </button>
      </div>
    </div>

    <div v-else class="tab-content tab-content-ai">
      <label class="field">
        <span>结构化提示词（作用于全部图片识别结果）</span>
        <textarea
          class="prompt-textarea"
          :value="prompt"
          placeholder="例如：先按图片编号总结，再给出跨图片综合结论。"
          @input="emit('update:prompt', ($event.target as HTMLTextAreaElement).value)"
        />
      </label>

      <div class="result-surface markdown-surface" v-html="renderedHtml" />

      <div class="button-row">
        <button class="secondary-button" type="button" :disabled="busy || !canCopy" @click="emit('copy-ai')">
          复制富文本
        </button>
        <button class="secondary-button" type="button" :disabled="busy || !canExportAi" @click="emit('export-ai')">
          导出 MD
        </button>
      </div>
    </div>
  </section>
</template>
