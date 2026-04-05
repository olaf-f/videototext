<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'

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

const textRef = ref<HTMLTextAreaElement | null>(null)

const lineCount = computed(() => {
  if (!props.modelValue.trim()) {
    return 0
  }

  return props.modelValue.split(/\r?\n/).length
})

const charCount = computed(() => props.modelValue.trim().length)

function resizeTextarea() {
  const el = textRef.value
  if (!el) {
    return
  }

  el.style.height = 'auto'
  el.style.height = `${Math.max(200, el.scrollHeight)}px`
}

function handleInput(event: Event) {
  const nextValue = (event.target as HTMLTextAreaElement).value
  emit('update:model-value', nextValue)
  resizeTextarea()
}

watch(
  () => props.modelValue,
  () => {
    void nextTick(() => resizeTextarea())
  },
)

onMounted(() => {
  resizeTextarea()
})
</script>

<template>
  <section class="panel accumulator-panel">
    <div class="panel-heading panel-heading-compact">
      <div>
        <p class="panel-kicker">结果归档</p>
        <h2>汇总编辑与导出</h2>
      </div>
      <button class="ghost-button" type="button" :disabled="busy" @click="emit('clear')">清空</button>
    </div>

    <div class="meta-row">
      <span class="meta-chip">共 {{ charCount }} 字</span>
      <span class="meta-chip">共 {{ lineCount }} 行</span>
    </div>

    <textarea
      ref="textRef"
      class="accumulator-textarea"
      :value="modelValue"
      placeholder="一键流程结果会自动写入这里，可继续编辑后导出。"
      @input="handleInput"
    />

    <div class="button-row">
      <button class="secondary-button" type="button" :disabled="busy || !canExport" @click="emit('export-md')">
        导出 Markdown
      </button>
      <button class="secondary-button" type="button" :disabled="busy || !canExport" @click="emit('export-txt')">
        导出文本
      </button>
    </div>
  </section>
</template>
