<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import Sortable, { type SortableEvent } from 'sortablejs'

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

const listRef = ref<HTMLElement | null>(null)
const dragging = ref(false)
let sortable: Sortable | null = null

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

function destroySortable() {
  if (sortable) {
    sortable.destroy()
    sortable = null
  }
}

function onSortEnd(event: SortableEvent) {
  dragging.value = false

  const fromIndex = event.oldIndex
  const toIndex = event.newIndex

  if (fromIndex == null || toIndex == null || fromIndex === toIndex) {
    return
  }

  emit('reorder-images', { fromIndex, toIndex })
}

function initSortable() {
  destroySortable()

  if (!listRef.value || props.imageCount < 2) {
    return
  }

  sortable = Sortable.create(listRef.value, {
    animation: 150,
    draggable: '.sortable-item',
    handle: '.drag-handle',
    forceFallback: true,
    fallbackTolerance: 3,
    ghostClass: 'is-sort-ghost',
    chosenClass: 'is-sort-chosen',
    dragClass: 'is-sort-drag',
    onStart: () => {
      dragging.value = true
    },
    onEnd: onSortEnd,
  })
}

function selectIndex(index: number) {
  if (dragging.value) {
    return
  }
  emit('set-index', index)
}

watch(
  () => [props.imageCount, props.activeImages.map((item) => item.id).join('|')] as const,
  async () => {
    await nextTick()
    initSortable()
  },
)

onBeforeUnmount(() => {
  destroySortable()
})
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
      <ul ref="listRef" class="sortable-list">
        <li
          v-for="(image, index) in activeImages"
          :key="image.id"
          class="sortable-item"
          :class="{ 'is-active': index === imageIndex }"
          @click="selectIndex(index)"
        >
          <span class="drag-handle" title="拖拽排序" aria-hidden="true">⋮⋮</span>
          <span class="sort-order">{{ index + 1 }}</span>
          <span class="sort-name">{{ image.displayName }}</span>
        </li>
      </ul>
    </div>
  </section>
</template>
