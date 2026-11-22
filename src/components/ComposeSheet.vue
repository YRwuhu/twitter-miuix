<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { MiuixBottomSheet, MiuixButton, MiuixText } from 'miuix-vue'
import AppIcon from './AppIcon.vue'
import AvatarBadge from './AvatarBadge.vue'
import { createTweet, toXApiError, uploadPhoto } from '../api/x'

const props = defineProps<{ modelValue: boolean; replyTo?: string; replyToId?: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: boolean]; publish: [text: string] }>()

const MAX = 280
const MAX_MEDIA = 4
const MAX_MEDIA_BYTES = 5 * 1024 * 1024
const ACCEPTED_TYPES = new Set(['image/jpeg', 'image/png', 'image/gif'])
const text = ref('')
const textarea = ref<HTMLTextAreaElement | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)
const media = ref<{ file: File; url: string; mediaId?: string }[]>([])
const uploading = ref(false)
const uploadError = ref('')
const composing = ref(false)

const remaining = computed(() => MAX - text.value.length)
const overLimit = computed(() => remaining.value < 0)
const canPublish = computed(() => (text.value.trim().length > 0 || media.value.length > 0) && !overLimit.value && !uploading.value)
const progress = computed(() => Math.min(text.value.length / MAX, 1))
const circumference = 2 * Math.PI * 9.5

const sheetOpen = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value),
})

watch(
  () => props.modelValue,
  async (open) => {
    if (open) {
      await nextTick()
      textarea.value?.focus()
    } else {
      text.value = ''
      clearMedia()
      uploadError.value = ''
    }
  },
)

function selectPhotos(event: Event): void {
  const input = event.target as HTMLInputElement
  const files = Array.from(input.files ?? [])
  input.value = ''
  uploadError.value = ''
  const available = MAX_MEDIA - media.value.length
  if (files.length > available) uploadError.value = `最多添加 ${MAX_MEDIA} 张图片`
  for (const file of files.slice(0, available)) {
    if (!ACCEPTED_TYPES.has(file.type)) {
      uploadError.value = '仅支持 JPEG、PNG 和 GIF 图片'
      continue
    }
    if (!file.size || file.size > MAX_MEDIA_BYTES) {
      uploadError.value = '每张图片不能超过 5 MiB'
      continue
    }
    media.value.push({ file, url: URL.createObjectURL(file) })
  }
}

function removePhoto(index: number): void {
  const [removed] = media.value.splice(index, 1)
  if (removed) URL.revokeObjectURL(removed.url)
}

function clearMedia(): void {
  for (const item of media.value) URL.revokeObjectURL(item.url)
  media.value = []
}

async function publish(): Promise<void> {
  if (!canPublish.value) return
  uploading.value = true
  uploadError.value = ''
  try {
    for (const item of media.value) {
      if (!item.mediaId) item.mediaId = await uploadPhoto(item.file)
    }
    await createTweet(text.value.trim(), props.replyToId, media.value.map((item) => item.mediaId!))
    emit('publish', text.value.trim())
    emit('update:modelValue', false)
    text.value = ''
    clearMedia()
  } catch (error) {
    uploadError.value = toXApiError(error).message
  } finally {
    uploading.value = false
  }
}

onBeforeUnmount(clearMedia)

function onInput(): void {
  if (!composing.value && text.value.length > MAX) {
    text.value = text.value.slice(0, MAX)
  }
}
</script>

<template>
  <MiuixBottomSheet
    v-model="sheetOpen"
    :title="replyTo ? '回复' : '新帖子'"
    class="ck-compose"
  >
    <template #start-action>
      <MiuixButton type="default" class="ck-compose__cancel" @click="sheetOpen = false">取消</MiuixButton>
    </template>
    <template #end-action>
      <MiuixButton type="primary" :disabled="!canPublish" class="ck-compose__publish" @click="publish">
        {{ uploading ? '上传中…' : '发布' }}
      </MiuixButton>
    </template>

    <div class="ck-compose__body">
      <div class="ck-compose__reply" v-if="replyTo">回复 {{ replyTo }}</div>

      <div class="ck-compose__main">
        <AvatarBadge name="我" :size="42" />
        <textarea
          ref="textarea"
          v-model="text"
          class="ck-compose__input"
          :placeholder="replyTo ? '发布你的回复' : '有什么新鲜事？'"
          rows="5"
          @compositionstart="composing = true"
          @compositionend="composing = false; onInput()"
          @input="onInput"
        />
      </div>

      <div v-if="media.length" class="ck-compose__media">
        <div v-for="(item, index) in media" :key="item.url" class="ck-compose__media-item">
          <img :src="item.url" :alt="item.file.name" />
          <button type="button" :aria-label="`移除图片 ${index + 1}`" @click="removePhoto(index)">
            <AppIcon name="close" :size="14" />
          </button>
        </div>
      </div>

      <span class="ck-compose__audience">
        <AppIcon name="globe" :size="14" />
        <span>所有人可回复</span>
      </span>

      <p v-if="uploadError" class="ck-compose__error" role="alert">{{ uploadError }}</p>

      <footer class="ck-compose__toolbar">
        <div class="ck-compose__tools">
          <button type="button" class="ck-compose__tool" aria-label="添加图片" :disabled="media.length >= MAX_MEDIA || uploading" @click="fileInput?.click()">
            <AppIcon name="image" :size="20" />
          </button>
          <input ref="fileInput" class="ck-compose__file-input" type="file" accept="image/jpeg,image/png,image/gif" multiple @change="selectPhotos" />
        </div>

        <div
          class="ck-compose__meter"
          :class="{ 'is-warning': remaining <= 20, 'is-over': overLimit }"
          :title="`还可输入 ${remaining} 个字符`"
        >
          <svg viewBox="0 0 24 24" width="24" height="24" aria-hidden="true">
            <circle cx="12" cy="12" r="9.5" fill="none" stroke="currentColor" stroke-opacity="0.16" stroke-width="2.2" />
            <circle
              cx="12"
              cy="12"
              r="9.5"
              fill="none"
              stroke="currentColor"
              stroke-width="2.2"
              stroke-linecap="round"
              :stroke-dasharray="`${progress * circumference} ${circumference}`"
              transform="rotate(-90 12 12)"
            />
          </svg>
          <MiuixText type="footnote1" class="ck-compose__count">{{ remaining }}</MiuixText>
        </div>
      </footer>
    </div>
  </MiuixBottomSheet>
</template>

<style scoped>
/* miuix BottomSheet 自带 28px 大圆角、拖拽手柄与弹簧入场。 */
.ck-compose :deep(.m-bottom-sheet__inner) {
  background: var(--ck-card);
  color: var(--ck-text);
}

.ck-compose__cancel,
.ck-compose__publish {
  min-height: 32px;
  height: 32px;
  padding: 0 14px;
  font-size: 13px;
}

.ck-compose__body {
  padding-bottom: 8px;
}

.ck-compose__reply {
  font-size: 12px;
  color: var(--ck-text-secondary);
  padding-bottom: 6px;
}

.ck-compose__main {
  display: flex;
  gap: 12px;
  padding: 4px 0 2px;
}

.ck-compose__input {
  flex: 1;
  min-width: 0;
  min-height: 118px;
  resize: none;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--ck-text);
  font-size: 17px;
  line-height: 23px;
  padding: 2px 0;
}
.ck-compose__input::placeholder {
  color: var(--ck-text-tertiary);
}

.ck-compose__audience {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  margin: 6px 0 0 54px;
  padding: 4px 10px;
  border: 0;
  border-radius: 9999px;
  background: var(--ck-active);
  color: var(--ck-blue);
  font-size: 11px;
  font-weight: 700;
}

.ck-compose__media {
  display: flex;
  gap: 8px;
  margin: 10px 0 0 54px;
  overflow-x: auto;
}
.ck-compose__media-item {
  position: relative;
  width: 74px;
  height: 74px;
  flex: none;
  overflow: hidden;
  border-radius: 10px;
  background: var(--ck-bg-sunken);
}
.ck-compose__media-item img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.ck-compose__media-item button {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 22px;
  height: 22px;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: 50%;
  background: rgba(0, 0, 0, 0.72);
  color: white;
  cursor: pointer;
}
.ck-compose__file-input {
  display: none;
}
.ck-compose__error {
  margin: 8px 0 0 54px;
  color: var(--ck-red);
  font-size: 12px;
}
.ck-compose__tool:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.ck-compose__toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px solid var(--ck-border);
}

.ck-compose__tools {
  display: flex;
  gap: 4px;
}
.ck-compose__tool {
  width: 34px;
  height: 34px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 9999px;
  background: transparent;
  color: var(--ck-blue);
  cursor: pointer;
}
.ck-compose__tool:hover {
  background: var(--ck-active);
}

.ck-compose__meter {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--ck-blue);
}
.ck-compose__meter.is-warning {
  color: #d9a400;
}
.ck-compose__meter.is-over {
  color: var(--ck-red);
}
.ck-compose__count {
  min-width: 22px;
  text-align: right;
  color: inherit;
  font-variant-numeric: tabular-nums;
}
</style>
