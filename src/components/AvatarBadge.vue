/** 真实头像：加载失败时回退到首字渐变。 */
<script setup lang="ts">
import { computed, ref, watch } from 'vue'

const props = withDefaults(
  defineProps<{
    name: string
    avatar?: string | null
    accent?: string
    size?: number
    square?: boolean
  }>(),
  { accent: '#1d9bf0', size: 40, square: false, avatar: null },
)

const failed = ref(false)

// 换了用户或换了 URL 时重置失败状态
watch(
  () => props.avatar,
  () => {
    failed.value = false
  },
)

const showImage = computed(() => !!props.avatar && !failed.value)
const initial = computed(() => props.name.trim().slice(0, 1).toUpperCase() || '?')

/** X 的头像 URL 带 `_normal` 后缀（48px）；换成 `_x96` 会更清晰。 */
const imageUrl = computed(() => {
  const url = props.avatar
  if (!url) return ''
  return url.replace(/_normal\.(jpg|jpeg|png|gif|webp)$/i, '_x96.$1')
})

function shade(hex: string, amount: number): string {
  const value = hex.replace('#', '')
  const num = Number.parseInt(
    value.length === 3 ? value.replace(/./g, (c) => c + c) : value,
    16,
  )
  const clamp = (v: number) => Math.max(0, Math.min(255, Math.round(v)))
  const r = clamp(((num >> 16) & 0xff) * (1 + amount))
  const g = clamp(((num >> 8) & 0xff) * (1 + amount))
  const b = clamp((num & 0xff) * (1 + amount))
  return `rgb(${r}, ${g}, ${b})`
}

const fallbackStyle = computed(() => ({
  background: `linear-gradient(140deg, ${props.accent} 0%, ${shade(props.accent, -0.28)} 100%)`,
  fontSize: `${Math.round(props.size * 0.42)}px`,
}))

const boxStyle = computed(() => ({
  width: `${props.size}px`,
  height: `${props.size}px`,
}))
</script>

<template>
  <span
    class="ck-avatar"
    :class="{ 'ck-avatar--square': square }"
    :style="boxStyle"
    :title="name"
  >
    <img
      v-if="showImage"
      :src="imageUrl"
      :alt="name"
      class="ck-avatar__img"
      loading="lazy"
      referrerpolicy="no-referrer"
      @error="failed = true"
    />
    <span v-else class="ck-avatar__fallback" :style="fallbackStyle">{{ initial }}</span>
  </span>
</template>

<style scoped>
.ck-avatar {
  position: relative;
  display: inline-flex;
  flex: none;
  align-items: center;
  justify-content: center;
  border-radius: 9999px;
  overflow: hidden;
  background: var(--ck-bg-sunken);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.14);
}
.ck-avatar--square {
  border-radius: 12px;
}
.ck-avatar__img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
.ck-avatar__fallback {
  width: 100%;
  height: 100%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-weight: 700;
  letter-spacing: 0.01em;
}
</style>