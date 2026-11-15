<script setup lang="ts">
import { computed, ref } from 'vue'
import AppIcon from './AppIcon.vue'
import AvatarBadge from './AvatarBadge.vue'
import MediaGrid from './MediaGrid.vue'
import VerifiedBadge from './VerifiedBadge.vue'
import { compactCount, relativeTime, type XPost } from '../types'

const props = defineProps<{ post: XPost }>()
const emit = defineEmits<{
  like: [id: string]
  repost: [id: string]
  bookmark: [id: string]
  reply: [id: string]
  open: [id: string]
}>()

const menuOpen = ref(false)

/** 转推时展示原帖作者与内容。 */
const display = computed<XPost>(() => props.post.reposted ?? props.post)
const isRepost = computed(() => !!props.post.reposted)

const likeLabel = computed(() => compactCount(display.value.metrics.like))
const repostLabel = computed(() => compactCount(display.value.metrics.repost))
const replyLabel = computed(() => compactCount(display.value.metrics.reply))
</script>

<template>
  <article class="ck-post-item">
    <div v-if="isRepost" class="ck-post__repost">
      <AppIcon name="repost" :size="13" />
      <span>{{ post.author.name }} 转推</span>
    </div>

    <article class="ck-post" @click="emit('open', post.id)">
      <AvatarBadge
        :name="display.author.name"
        :avatar="display.author.avatar"
        :size="42"
      />

      <div class="ck-post__body">
        <header class="ck-post__head">
          <span class="ck-post__name">{{ display.author.name }}</span>
          <VerifiedBadge :verified="display.author.verified" :size="14" />
          <span class="ck-post__handle">@{{ display.author.handle }}</span>
          <span class="ck-post__dot">·</span>
          <time class="ck-post__time">{{ relativeTime(display.createdAt) }}</time>
          <button
            class="ck-post__more"
            type="button"
            aria-label="更多操作"
            @click.stop="menuOpen = !menuOpen"
          >
            <AppIcon name="more" :size="17" />
          </button>
        </header>

        <p v-if="display.inReplyToHandle" class="ck-post__reply-to">
          回复 @{{ display.inReplyToHandle }}
        </p>

        <p class="ck-post__text">{{ display.text }}</p>

        <MediaGrid v-if="display.media?.length" :media="display.media" />

        <!-- 引用帖 -->
        <blockquote v-if="display.quoted" class="ck-post__quote">
          <div class="ck-post__quote-head">
            <AvatarBadge
              :name="display.quoted.author.name"
              :avatar="display.quoted.author.avatar"
              :size="18"
            />
            <span class="ck-post__quote-name">{{ display.quoted.author.name }}</span>
            <VerifiedBadge :verified="display.quoted.author.verified" :size="11" />
            <span class="ck-post__quote-handle">@{{ display.quoted.author.handle }}</span>
          </div>
          <p class="ck-post__quote-text">{{ display.quoted.text }}</p>
        </blockquote>

        <footer class="ck-post__actions">
          <button
            class="ck-action"
            type="button"
            aria-label="回复"
            @click.stop="emit('reply', post.id)"
          >
            <span class="ck-action__icon"><AppIcon name="comment" :size="17" /></span>
            <span class="ck-action__count">{{ replyLabel }}</span>
          </button>

          <button
            class="ck-action ck-action--repost"
            :class="{ 'is-active': display.isReposted }"
            type="button"
            aria-label="转推"
            @click.stop="emit('repost', post.id)"
          >
            <span class="ck-action__icon"><AppIcon name="repost" :size="17" /></span>
            <span class="ck-action__count">{{ repostLabel }}</span>
          </button>

          <button
            class="ck-action ck-action--like"
            :class="{ 'is-active': display.isLiked }"
            type="button"
            aria-label="喜欢"
            @click.stop="emit('like', post.id)"
          >
            <span class="ck-action__icon">
              <AppIcon name="heart" :size="17" :filled="display.isLiked" />
            </span>
            <span class="ck-action__count">{{ likeLabel }}</span>
          </button>

          <button
            class="ck-action ck-action--bookmark"
            :class="{ 'is-active': display.isBookmarked }"
            type="button"
            aria-label="书签"
            @click.stop="emit('bookmark', post.id)"
          >
            <span class="ck-action__icon">
              <AppIcon name="bookmark" :size="16" :filled="display.isBookmarked" />
            </span>
          </button>

          <button class="ck-action" type="button" aria-label="分享" @click.stop>
            <span class="ck-action__icon"><AppIcon name="share" :size="16" /></span>
          </button>
        </footer>

        <Transition name="ck-menu">
          <div v-if="menuOpen" class="ck-post__menu" @click.stop>
            <button type="button" @click="menuOpen = false">复制帖子链接</button>
            <button type="button" @click="menuOpen = false">不感兴趣</button>
            <button type="button" @click="menuOpen = false">举报帖子</button>
          </div>
        </Transition>
      </div>
    </article>
  </article>
</template>

<style scoped>
/* 平面列表行：HyperOS 的时间线不做卡片，靠留白与分隔线组织 */
.ck-post-item {
  position: relative;
  padding: 16px 16px 14px;
  border-bottom: 1px solid var(--ck-divider);
  cursor: pointer;
  transition: background-color 120ms linear;
}
.ck-post-item:hover {
  background: var(--ck-hover);
}
.ck-post-item:active {
  background: var(--ck-hover);
}

.ck-post__repost {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: -2px 0 8px 54px;
  font-size: 12px;
  font-weight: 600;
  color: var(--ck-text-tertiary);
}

.ck-post {
  display: flex;
  gap: 12px;
}

.ck-post__body {
  flex: 1;
  min-width: 0;
}

.ck-post__head {
  display: flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  line-height: 19px;
}

.ck-post__name {
  font-size: 15px;
  font-weight: 700;
  letter-spacing: -0.01em;
  color: var(--ck-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 42%;
}

.ck-post__handle {
  font-size: 14px;
  color: var(--ck-text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
  flex-shrink: 1;
}

.ck-post__dot,
.ck-post__time {
  font-size: 14px;
  color: var(--ck-text-tertiary);
  white-space: nowrap;
  flex: none;
}

.ck-post__more {
  margin-left: auto;
  flex: none;
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 9999px;
  background: transparent;
  color: var(--ck-text-tertiary);
  cursor: pointer;
}
.ck-post__more:hover {
  background: var(--ck-bg-sunken);
  color: var(--ck-text);
}

.ck-post__reply-to {
  margin: 3px 0 0;
  font-size: 13px;
  color: var(--ck-text-tertiary);
}

.ck-post__text {
  margin: 4px 0 0;
  font-size: 15px;
  line-height: 21px;
  letter-spacing: -0.005em;
  color: var(--ck-text);
  white-space: pre-wrap;
  word-break: break-word;
}

/* 引用帖：内嵌块，用淡底而不是边框 */
.ck-post__quote {
  margin: 10px 0 0;
  padding: 10px 12px;
  border-radius: 14px;
  background: var(--ck-bg-sunken);
}
.ck-post__quote-head {
  display: flex;
  align-items: center;
  gap: 5px;
}
.ck-post__quote-name {
  font-size: 13px;
  font-weight: 700;
  color: var(--ck-text);
}
.ck-post__quote-handle {
  font-size: 13px;
  color: var(--ck-text-tertiary);
}
.ck-post__quote-text {
  margin: 5px 0 0;
  font-size: 14px;
  line-height: 19px;
  color: var(--ck-text);
  white-space: pre-wrap;
  word-break: break-word;
}

/* 操作栏：等距分布，图标 18px，计数轻量 */
.ck-post__actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin: 10px -6px -4px;
  max-width: 320px;
}

.ck-action {
  display: inline-flex;
  align-items: center;
  border: 0;
  background: transparent;
  color: var(--ck-text-tertiary);
  cursor: pointer;
  padding: 0;
  border-radius: 9999px;
  transition: color 140ms linear;
}

.ck-action__icon {
  width: 30px;
  height: 30px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 9999px;
  transition: background-color 140ms linear;
}
.ck-action__count {
  font-size: 12px;
  min-width: 10px;
  padding-right: 5px;
  color: inherit;
  font-variant-numeric: tabular-nums;
}

.ck-action:hover {
  color: var(--ck-blue);
}
.ck-action:hover .ck-action__icon {
  background: var(--ck-active);
}

.ck-action--repost:hover,
.ck-action--repost.is-active {
  color: var(--ck-green);
}
.ck-action--repost.is-active .ck-action__icon {
  background: rgba(0, 186, 124, 0.12);
}

.ck-action--like:hover,
.ck-action--like.is-active {
  color: var(--ck-pink);
}
.ck-action--like.is-active .ck-action__icon {
  background: rgba(249, 24, 128, 0.12);
}

.ck-action--bookmark:hover,
.ck-action--bookmark.is-active {
  color: var(--ck-blue);
}

/* 更多菜单 */
.ck-post__menu {
  position: absolute;
  right: 14px;
  top: 44px;
  z-index: 8;
  min-width: 172px;
  padding: 6px;
  border-radius: 16px;
  background: var(--ck-bg-elevated);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.22);
}
.ck-post__menu button {
  display: block;
  width: 100%;
  text-align: left;
  padding: 10px 12px;
  border: 0;
  border-radius: 10px;
  background: transparent;
  color: var(--ck-text);
  font-size: 14px;
  cursor: pointer;
}
.ck-post__menu button:hover {
  background: var(--ck-bg-sunken);
}

.ck-menu-enter-active,
.ck-menu-leave-active {
  transition: opacity 120ms linear, transform 140ms var(--ck-ease-out);
}
.ck-menu-enter-from,
.ck-menu-leave-to {
  opacity: 0;
  transform: translateY(-6px) scale(0.97);
}
</style>
