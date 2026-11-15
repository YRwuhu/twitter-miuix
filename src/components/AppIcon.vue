<script setup lang="ts">
import { computed } from 'vue'
import ICONS from './icons'

const props = withDefaults(
  defineProps<{
    name: string
    size?: number
    filled?: boolean
    strokeWidth?: number
  }>(),
  { size: 24, filled: false, strokeWidth: 1.9 },
)

const glyph = computed(() => {
  const filledName = props.filled ? `${props.name}Fill` : ''
  return (filledName && ICONS[filledName]) || ICONS[props.name] || ''
})
</script>

<template>
  <svg
    class="ck-icon"
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    aria-hidden="true"
    focusable="false"
    :stroke-width="strokeWidth"
  >
    <g v-html="glyph" />
  </svg>
</template>

<style scoped>
.ck-icon {
  display: block;
  flex: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
  overflow: visible;
}
</style>