<script setup lang="ts">
import { computed } from 'vue'
import { MiuixText } from 'miuix-vue'
import AppIcon from '../components/AppIcon.vue'
import AvatarBadge from '../components/AvatarBadge.vue'
import VerifiedBadge from '../components/VerifiedBadge.vue'
import MediaGrid from '../components/MediaGrid.vue'
import TweetCard from '../components/TweetCard.vue'
import { compactCount, relativeTime, type XPost } from '../types'

const props = defineProps<{
  post: XPost | null
  replies: XPost[]
  loading: boolean
}>()
const emit = defineEmits<{
  back: []
  like: [id: string]
  repost: [id: string]
  bookmark: [id: string]
  reply: [id: string]
  open: [id: string]
}>()

/** 详情页展示的正文（转推时用原帖）。 */
const display = computed<XPost | null>(() => props.post?.reposted ?? props.post ?? null)

const likeLabel = computed(() => compactCount(display.value?.metrics.like ?? 0))
const repostLabel = computed(() => compactCount(display.value?.metrics.repost ?? 0))
const replyLabel = computed(() => compactCount(display.value?.metrics.reply ?? 0))
const viewLabel = computed(() => compactCount(display.value?.metrics.view ?? 0))
</script>

<template>
  <div class="ck-detail">
    <header class="ck-detail__bar">
      <button type="button" class="ck-detail__back" aria-label="返回" @click="emit('back')">
        <AppIcon name="arrowLeft" :size="21" />
      </button>
      <MiuixText type="title3" weight="medium" class="ck-detail__bar-title">帖子</MiuixText>
      <span class="ck-detail__bar-spacer" />
    </header>

    <div v-if="display" class="ck-detail__scroll">
      <div class="ck-detail__content">
        <article class="ck-detail__main">
          <div class="ck-detail__author">
            <AvatarBadge
              :name="display.author.name"
              :avatar="display.author.avatar"
              :size="44"
            />
            <div class="ck-detail__author-meta">
              <div class="ck-detail__author-name">
                <strong>{{ display.author.name }}</strong>
                <VerifiedBadge :verified="display.author.verified" :size="14" />
              </div>
              <span class="ck-detail__handle">@{{ display.author.handle }}</span>
            </div>
          </div>

          <p v-if="display.inReplyToHandle" class="ck-detail__reply-to">
            回复 @{{ display.inReplyToHandle }}
          </p>

          <p class="ck-detail__text">{{ display.text }}</p>

          <MediaGrid v-if="display.media?.length" :media="display.media" large />

          <!-- 引用帖 -->
          <blockquote v-if="display.quoted" class="ck-detail__quote">
            <div class="ck-detail__quote-head">
              <AvatarBadge
                :name="display.quoted.author.name"
                :avatar="display.quoted.author.avatar"
                :size="20"
              />
              <span class="ck-detail__quote-name">{{ display.quoted.author.name }}</span>
              <VerifiedBadge :verified="display.quoted.author.verified" :size="11" />
              <span class="ck-detail__quote-handle">@{{ display.quoted.author.handle }}</span>
            </div>
            <p class="ck-detail__quote-text">{{ display.quoted.text }}</p>
            <MediaGrid v-if="display.quoted.media?.length" :media="display.quoted.media" />
          </blockquote>

          <div class="ck-detail__time">
            <span>{{ relativeTime(display.createdAt) }}</span>
            <span v-if="viewLabel">{{ viewLabel }} 次查看</span>
          </div>

          <div class="ck-detail__stats">
            <span><strong>{{ replyLabel || '0' }}</strong> 回复</span>
            <span><strong>{{ repostLabel || '0' }}</strong> 转推</span>
            <span><strong>{{ likeLabel || '0' }}</strong> 喜欢</span>
            <span><strong>{{ compactCount(display.metrics.bookmark) || '0' }}</strong> 书签</span>
          </div>

          <div class="ck-detail__actions">
            <button type="button" aria-label="回复" @click="emit('reply', post!.id)">
              <AppIcon name="comment" :size="18" />
            </button>
            <button
              type="button"
              :class="{ 'is-reposted': display.isReposted }"
              aria-label="转推"
              @click="emit('repost', post!.id)"
            >
              <AppIcon name="repost" :size="18" />
            </button>
            <button
              type="button"
              :class="{ 'is-liked': display.isLiked }"
              aria-label="喜欢"
              @click="emit('like', post!.id)"
            >
              <AppIcon name="heart" :size="18" :filled="display.isLiked" />
            </button>
            <button
              type="button"
              :class="{ 'is-bookmarked': display.isBookmarked }"
              aria-label="书签"
              @click="emit('bookmark', post!.id)"
            >
              <AppIcon name="bookmark" :size="17" :filled="display.isBookmarked" />
            </button>
            <button type="button" aria-label="分享">
              <AppIcon name="share" :size="17" />
            </button>
          </div>
        </article>

        <button type="button" class="ck-detail__reply" @click="emit('reply', post!.id)">
          <span class="ck-detail__reply-prompt">发布你的回复</span>
        </button>

        <div v-if="loading" class="ck-detail__loading">正在加载回复…</div>

        <TweetCard
          v-for="reply in replies"
          :key="reply.id"
          :post="reply"
          @like="emit('like', $event)"
          @repost="emit('repost', $event)"
          @bookmark="emit('bookmark', $event)"
          @reply="emit('reply', $event)"
          @open="emit('open', $event)"
        />

        <p v-if="!loading && !replies.length" class="ck-detail__end">还没有回复</p>
      </div>
    </div>

    <div v-else class="ck-detail__empty">
      <MiuixText type="body2" color="var(--ck-text-secondary)">帖子不存在或已删除。</MiuixText>
    </div>
  </div>
</template>

<style scoped>
.ck-detail {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--ck-bg);
}

.ck-detail__bar {
  position: relative;
  flex: none;
  justify-content: space-between;
  display: flex;
  align-items: center;
  height: 56px;
  padding: 0 8px;
}
.ck-detail__back {
  width: 34px;
  height: 34px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 9999px;
  background: transparent;
  color: var(--ck-text);
  cursor: pointer;
}
.ck-detail__back:hover {
  background: var(--ck-hover);
}
.ck-detail__bar-title {  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  max-width: 56%;
  text-align: center;
  pointer-events: none;
  color: var(--ck-text);
}
.ck-detail__bar-spacer {
  width: 34px;
  flex: none;
}

.ck-detail__scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  scrollbar-width: none;
}
.ck-detail__scroll::-webkit-scrollbar {
  width: 0;
}

.ck-detail__content {
  display: flex;
  flex-direction: column;
  padding: 0;
}

.ck-detail__main {
  padding: 16px;
  border-bottom: 1px solid var(--ck-divider);
}

.ck-detail__author {
  display: flex;
  align-items: center;
  gap: 10px;
}
.ck-detail__author-meta {
  flex: 1;
  min-width: 0;
}
.ck-detail__author-name {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 15px;
  font-weight: 700;
  color: var(--ck-text);
}
.ck-detail__handle {
  display: block;
  font-size: 12px;
  color: var(--ck-text-secondary);
}

.ck-detail__reply-to {
  margin: 10px 0 0;
  font-size: 13px;
  color: var(--ck-text-secondary);
}

.ck-detail__text {
  margin: 10px 0 0;
  font-size: 19px;
  line-height: 26px;
  color: var(--ck-text);
  white-space: pre-wrap;
  word-break: break-word;
}

.ck-detail__quote {
  margin: 12px 0 0;
  padding: 10px 12px;
  border-radius: 12px;
  background: var(--ck-bg-sunken);
}
.ck-detail__quote-head {
  display: flex;
  align-items: center;
  gap: 5px;
}
.ck-detail__quote-name {
  font-size: 13px;
  font-weight: 700;
  color: var(--ck-text);
}
.ck-detail__quote-handle {
  font-size: 13px;
  color: var(--ck-text-secondary);
}
.ck-detail__quote-text {
  margin: 5px 0 0;
  font-size: 14px;
  line-height: 19px;
  color: var(--ck-text);
  white-space: pre-wrap;
  word-break: break-word;
}

.ck-detail__time {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
  font-size: 12px;
  color: var(--ck-text-secondary);
}

.ck-detail__stats {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  margin-top: 10px;
  padding: 10px 0;
  border-top: 1px solid var(--ck-border);
  border-bottom: 1px solid var(--ck-border);
  font-size: 12px;
  color: var(--ck-text-secondary);
}
.ck-detail__stats strong {
  color: var(--ck-text);
  font-weight: 700;
}

.ck-detail__actions {
  display: flex;
  justify-content: space-between;
  padding-top: 8px;
}
.ck-detail__actions button {
  width: 36px;
  height: 36px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 9999px;
  background: transparent;
  color: var(--ck-text-tertiary);
  cursor: pointer;
}
.ck-detail__actions button:hover {
  color: var(--ck-blue);
  background: var(--ck-active);
}
.ck-detail__actions button.is-liked {
  color: var(--ck-pink);
}
.ck-detail__actions button.is-reposted {
  color: var(--ck-green);
}
.ck-detail__actions button.is-bookmarked {
  color: var(--ck-blue);
}

.ck-detail__reply {
  display: flex;
  align-items: center;
  width: 100%;
  padding: 12px 16px;
  border: 0;
  border-bottom: 1px solid var(--ck-divider);
  background: transparent;
  cursor: text;
}
.ck-detail__reply-prompt {
  flex: 1;
  height: 32px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  border-radius: 9999px;
  background: var(--ck-bg-sunken);
  color: var(--ck-text-tertiary);
  font-size: 13px;
}

.ck-detail__loading {
  padding: 18px 0 4px;
  text-align: center;
  font-size: 12px;
  color: var(--ck-text-secondary);
}

.ck-detail__end {
  margin: 0;
  padding: 16px 0 4px;
  text-align: center;
  font-size: 11px;
  color: var(--ck-text-tertiary);
}

.ck-detail__empty {
  flex: 1;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 40px;
}
</style>