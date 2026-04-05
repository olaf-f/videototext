<script setup lang="ts">
import { computed } from 'vue'

import { renderMarkdown } from '../lib/markdown'
import type { AiResult } from '../lib/types'

const props = defineProps<{
  aiResult: AiResult | null
  busy: boolean
  canCopy: boolean
  canExport: boolean
  prompt: string
}>()

const emit = defineEmits<{
  copy: []
  export: []
  'update:prompt': [value: string]
}>()

const renderedHtml = computed(() => renderMarkdown(props.aiResult?.markdown ?? ''))
</script>

<template>
  <section class="panel result-panel">
    <div class="panel-heading">
      <div>
        <p class="panel-kicker">流程输出</p>
        <h2>AI 结构化结果</h2>
      </div>
      <span v-if="aiResult" class="result-meta">耗时 {{ aiResult.durationMs }} ms</span>
    </div>

    <label class="field">
      <span>结构化提示词</span>
      <textarea
        class="prompt-textarea"
        :value="prompt"
        placeholder="例如：按标题、摘要、关键字段分段输出。"
        @input="emit('update:prompt', ($event.target as HTMLTextAreaElement).value)"
      />
    </label>

    <div class="markdown-surface" v-html="renderedHtml" />

    <div class="button-row">
      <button class="secondary-button" type="button" :disabled="busy || !canCopy" @click="emit('copy')">
        复制富文本
      </button>
      <button class="secondary-button" type="button" :disabled="busy || !canExport" @click="emit('export')">
        导出 MD
      </button>
    </div>
  </section>
</template>
