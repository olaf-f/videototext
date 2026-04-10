<script setup lang="ts">
import { computed, ref, watch } from 'vue'

const props = defineProps<{
  busy: boolean
  canExport: boolean
  modelValue: string
}>()

const emit = defineEmits<{
  clear: []
  'export-md': []
  'export-txt': []
  'update:model-value': [value: string]
}>()

const draftValue = ref(props.modelValue)
const editing = ref(false)

const lineCount = computed(() => {
  if (!draftValue.value.trim()) {
    return 0
  }

  return draftValue.value.split(/\r?\n/).length
})

const charCount = computed(() => draftValue.value.trim().length)

watch(
  () => props.modelValue,
  (value) => {
    if (!editing.value) {
      draftValue.value = value
    }
  },
)

function syncDraftToStore() {
  if (draftValue.value !== props.modelValue) {
    emit('update:model-value', draftValue.value)
  }
}

function handleInput(event: Event) {
  draftValue.value = (event.target as HTMLTextAreaElement).value
}

function handleFocus() {
  editing.value = true
}

function handleBlur() {
  editing.value = false
  syncDraftToStore()
}

function handleMouseLeave() {
  if (editing.value) {
    syncDraftToStore()
  }
}

function handleClear() {
  draftValue.value = ''
  editing.value = false
  emit('clear')
}

function exportMarkdown() {
  syncDraftToStore()
  emit('export-md')
}

function exportText() {
  syncDraftToStore()
  emit('export-txt')
}
</script>

<template>
  <section class="panel accumulator-panel">
    <div class="panel-heading panel-heading-compact">
      <div>
        <p class="panel-kicker">结果归档</p>
        <h2>汇总编辑与导出</h2>
      </div>
      <button class="ghost-button" type="button" :disabled="busy" @click="handleClear">清空</button>
    </div>

    <div class="meta-row">
      <span class="meta-chip">共 {{ charCount }} 字</span>
      <span class="meta-chip">共 {{ lineCount }} 行</span>
    </div>

    <textarea
      class="accumulator-textarea"
      :value="draftValue"
      placeholder="每次 AI 结构化结果会自动写入这里，可继续编辑后导出。"
      @focus="handleFocus"
      @blur="handleBlur"
      @mouseleave="handleMouseLeave"
      @input="handleInput"
    />

    <div class="button-row">
      <button class="secondary-button" type="button" :disabled="busy || !canExport" @click="exportMarkdown">
        导出 Markdown
      </button>
      <button class="secondary-button" type="button" :disabled="busy || !canExport" @click="exportText">
        导出文本
      </button>
    </div>
  </section>
</template>
