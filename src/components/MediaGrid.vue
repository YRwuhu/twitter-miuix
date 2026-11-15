<script setup lang="ts">
/**
 * 真实媒体渲染：图片 / 视频 / 动图。
 *
 * X 的图片 URL 支持尺寸后缀（`?name=small|medium|large|orig`）。
 * 视频取 `variants` 里码率最高的一条。
 */
import { computed, ref } from 'vue'
import type { XMedia } from '../types'

const props = withDefaults(
  defineProps<{
    media: XMedia[]
    /** 详情页用大图 */
    large?: boolean
  }>(),
  { large: false },
)

const failed = ref<Set<number>>(new Set())

function markFailed(index: number): void {
  failed.value = new Set(failed.value).add(index)
}

function imageSrc(item: XMedia): string {
  if (!item.url) return ''
  const name = props.large ? 'large' : 'medium'
  const base = item.url.replace(/\?.*$/, '')
  return `${base}?name=${name}`
}

function videoSrc(item: XMedia): string {
  const variant = item.variants?.[0]
  return variant?.url ?? ''
}

const layoutClass = computed(() => {
  const count = Math.min(props.media.length, 4)
  return `ck-media--${count}`
})

function aspectStyle(item: XMedia): Record<string, string> {
  if (props.large) {
    return { aspectRatio: item.width && item.height ? `${item.width} / ${item.height}` : '16 / 9' }
  }
  // 网格里限制过高的图，避免单张图撑满整屏
  return { aspectRatio: item.width && item.height ? `${item.width} / ${item.height}` : '16 / 9' }
}
</script>

<template>
  <div v-if="media.length" class="ck-media" :class="layoutClass">
    <div v-for="(item, index) in media" :key="index" class="ck-media__item" :style="aspectStyle(item)">
      <!-- 视频 -->
      <video
        v-if="item.kind === 'video' && videoSrc(item)"
        class="ck-media__video"
        :src="videoSrc(item)"
        :poster="item.preview || undefined"
        controls
        playsinline
        preload="metadata"
        referrerpolicy="no-referrer"
      />

      <!-- 动图：静音循环自动播放 -->
      <video
        v-else-if="item.kind === 'animated_gif' && videoSrc(item)"
        class="ck-media__video"
        :src="videoSrc(item)"
        :poster="item.preview || undefined"
        autoplay
        loop
        muted
        playsinline
        referrerpolicy="no-referrer"
      />

      <!-- 图片 -->
      <img
        v-else-if="!failed.has(index)"
        :src="imageSrc(item)"
        :alt="item.altText || '图片'"
        class="ck-media__img"
        loading="lazy"
        referrerpolicy="no-referrer"
        @error="markFailed(index)"
      />

      <!-- 加载失败占位 -->
      <div v-else class="ck-media__broken">图片加载失败</div>

      <span v-if="item.altText" class="ck-media__alt" :title="item.altText">ALT</span>
      <span v-if="item.durationMs" class="ck-media__duration">
        {{ Math.floor(item.durationMs / 1000) }}s
      </span>
    </div>
  </div>
</template>

<style scoped>
.ck-media {
  display: grid;
  gap: 3px;
  margin-top: 10px;
  border-radius: 12px;
  overflow: hidden;
}
.ck-media--1 {
  grid-template-columns: 1fr;
}
.ck-media--2 {
  grid-template-columns: 1fr 1fr;
}
.ck-media--3,
.ck-media--4 {
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
}
.ck-media--3 .ck-media__item:first-child {
  grid-row: span 2;
}

.ck-media__item {
  position: relative;
  min-height: 88px;
  max-height: 420px;
  overflow: hidden;
  background: var(--ck-bg-sunken);
  display: flex;
  align-items: center;
  justify-content: center;
}

.ck-media__img,
.ck-media__video {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.ck-media__broken {
  font-size: 11px;
  color: var(--ck-text-tertiary);
}

.ck-media__alt {
  position: absolute;
  left: 6px;
  bottom: 6px;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.72);
  color: #fff;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.04em;
}

.ck-media__duration {
  position: absolute;
  right: 6px;
  bottom: 6px;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.72);
  color: #fff;
  font-size: 10px;
  font-variant-numeric: tabular-nums;
}
</style>