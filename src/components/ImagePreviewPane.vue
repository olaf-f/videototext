<script setup lang="ts">
import { computed, ref } from 'vue'

import type { ActiveImageSource } from '../lib/types'

const props = defineProps<{
  activeImage: ActiveImageSource | null
  activeImages: ActiveImageSource[]
  imageCount: number
  imageIndex: number
}>()

const emit = defineEmits<{
  'set-index': [index: number]
  'reorder-images': [payload: { fromIndex: number; toIndex: number }]
}>()

const dragFromIndex = ref<number | null>(null)

const positionLabel = computed(() => {
  if (!props.imageCount) {
    return '0/0'
  }

  return `${props.imageIndex + 1}/${props.imageCount}`
})

function toPrev() {
  emit('set-index', props.imageIndex - 1)
}

function toNext() {
  emit('set-index', props.imageIndex + 1)
}

function onDragStart(index: number, event: DragEvent) {
  dragFromIndex.value = index
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.dropEffect = 'move'
    event.dataTransfer.setData('text/plain', String(index))
  }
}

function onDragOver(event: DragEvent) {
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
}

function onDrop(toIndex: number, event: DragEvent) {
  event.preventDefault()
  const fromText = event.dataTransfer?.getData('text/plain') ?? ''
  const parsed = Number.parseInt(fromText, 10)
  const fromIndex = Number.isFinite(parsed) ? parsed : dragFromIndex.value
  dragFromIndex.value = null

  if (fromIndex == null || fromIndex === toIndex) {
    return
  }

  emit('reorder-images', { fromIndex, toIndex })
}

function onDragEnd() {
  dragFromIndex.value = null
}
</script>

<template>
  <section class="panel preview-panel">
    <div class="panel-heading panel-heading-compact">
      <div>
        <p class="panel-kicker">图像预览</p>
        <h2>预览窗口</h2>
      </div>
      <span class="meta-chip">{{ positionLabel }}</span>
    </div>

    <div v-if="activeImage" class="preview-frame">
      <img :alt="activeImage.displayName" :src="activeImage.previewUrl" class="preview-image" />
    </div>
    <div v-else class="empty-state">
      <p>上传图片后会在这里展示。</p>
    </div>

    <div v-if="imageCount > 1" class="button-row">
      <button class="secondary-button" type="button" :disabled="imageIndex <= 0" @click="toPrev">上一张</button>
      <button class="secondary-button" type="button" :disabled="imageIndex >= imageCount - 1" @click="toNext">下一张</button>
    </div>

    <div v-if="imageCount > 1" class="sortable-list-wrap">
      <p class="field-hint">拖拽排序（OCR/AI 按此顺序执行）</p>
      <ul class="sortable-list">
        <li
          v-for="(image, index) in activeImages"
          :key="image.id"
          class="sortable-item"
          :class="{ 'is-active': index === imageIndex }"
          draggable="true"
          @click="emit('set-index', index)"
          @dragstart="onDragStart(index, $event)"
          @dragover="onDragOver"
          @drop="onDrop(index, $event)"
          @dragend="onDragEnd"
        >
          <span class="sort-order">{{ index + 1 }}</span>
          <span class="sort-name">{{ image.displayName }}</span>
        </li>
      </ul>
    </div>
  </section>
</template>
