<script setup lang="ts">
import AppIcon from './AppIcon.vue'

withDefaults(defineProps<{ verified?: boolean; size?: number }>(), { verified: false, size: 15 })
</script>

<template>
  <span v-if="verified" class="ck-verified" :style="{ width: `${size}px`, height: `${size}px` }">
    <AppIcon name="check" :size="Math.round(size * 0.66)" :stroke-width="3.2" />
  </span>
</template>

<style scoped>
/* X 的认证徽章：蓝色实心圆 + 白色对勾。 */
.ck-verified {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: none;
  border-radius: 9999px;
  background: var(--ck-blue);
  color: #fff;
  vertical-align: middle;
}
</style>