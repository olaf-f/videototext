<script setup lang="ts">
import { computed } from 'vue'

import type { ActiveImageSource, AppSettings } from '../lib/types'

const props = defineProps<{
  activeImage: ActiveImageSource | null
  busyLabel: string
  errorMessage: string
  imageCount: number
  settings: AppSettings
  statusMessage: string
}>()

const sourceLabel = computed(() => {
  if (!props.activeImage) {
    return '未导入'
  }

  switch (props.activeImage.sourceType) {
    case 'file':
      return '本地文件'
    case 'clipboard':
      return '剪贴板'
    case 'url':
      return '图片 URL'
    default:
      return props.activeImage.sourceType
  }
})
</script>

<template>
  <section class="status-bar">
    <div class="status-group">
      <span class="status-pill" :data-state="imageCount > 0 ? 'ready' : 'empty'">
        {{ imageCount > 0 ? `已选图片：${imageCount} 张（当前：${sourceLabel}）` : '图片状态：未导入' }}
      </span>
      <span class="status-pill" :data-state="settings.deepseekApiKeySaved ? 'ready' : 'empty'">
        {{ settings.deepseekApiKeySaved ? 'AI 服务：已配置' : 'AI 服务：未配置密钥' }}
      </span>
    </div>

    <p class="status-copy">
      <span v-if="busyLabel">{{ busyLabel }}</span>
      <span v-else-if="errorMessage" class="status-error">{{ errorMessage }}</span>
      <span v-else>{{ statusMessage || '系统已就绪。' }}</span>
    </p>
  </section>
</template>
