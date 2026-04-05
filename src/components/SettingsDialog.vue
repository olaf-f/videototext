<script setup lang="ts">
import { reactive, watch } from 'vue'

import type { AppSettings, SaveSettingsPayload } from '../lib/types'

const props = defineProps<{
  busy: boolean
  open: boolean
  settings: AppSettings
}>()

const emit = defineEmits<{
  close: []
  save: [payload: SaveSettingsPayload]
}>()

const form = reactive<SaveSettingsPayload>({
  deepseekApiKey: '',
  defaultPrompt: '',
  webPortalUrl: '',
})

watch(
  () => [props.open, props.settings] as const,
  ([open, value]) => {
    if (!open) {
      return
    }

    form.deepseekApiKey = ''
    form.defaultPrompt = value.defaultPrompt
    form.webPortalUrl = value.webPortalUrl
  },
  { immediate: true },
)

function submit() {
  emit('save', {
    deepseekApiKey: form.deepseekApiKey,
    defaultPrompt: form.defaultPrompt,
    webPortalUrl: form.webPortalUrl,
  })
}
</script>

<template>
  <div v-if="open" class="modal-backdrop" @click.self="emit('close')">
    <section class="modal-card">
      <div class="panel-heading">
        <div>
          <p class="panel-kicker">系统设置</p>
          <h2>应用参数</h2>
        </div>
        <button class="ghost-button" type="button" :disabled="busy" @click="emit('close')">关闭</button>
      </div>

      <div class="settings-grid">
        <label class="field">
          <span>DeepSeek API Key 状态</span>
          <div class="status-pill" :data-state="settings.deepseekApiKeySaved ? 'ready' : 'empty'">
            {{ settings.deepseekApiKeySaved ? '后端已保存密钥' : '未检测到已保存密钥' }}
          </div>
        </label>

        <label class="field">
          <span>DeepSeek API Key</span>
          <input
            v-model="form.deepseekApiKey"
            class="text-input"
            placeholder="输入后点击“保存设置”，无需重启立即生效"
            type="password"
          />
          <small class="field-hint">为空时不覆盖现有密钥。</small>
        </label>

        <label class="field">
          <span>默认提示词</span>
          <textarea v-model="form.defaultPrompt" class="prompt-textarea" placeholder="用于 AI 结构化的默认提示词。" />
        </label>

        <label class="field">
          <span>网页 OCR 门户地址</span>
          <input
            v-model="form.webPortalUrl"
            class="text-input"
            placeholder="https://example.com/web-ocr"
            type="url"
          />
        </label>
      </div>

      <div class="button-row">
        <button class="secondary-button" type="button" :disabled="busy" @click="emit('close')">取消</button>
        <button class="primary-button" type="button" :disabled="busy" @click="submit">
          {{ busy ? '保存中...' : '保存设置' }}
        </button>
      </div>
    </section>
  </div>
</template>
