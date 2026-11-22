<script setup lang="ts">
/** 书签页：真实书签时间线，滚动加载。 */
import { onMounted, onUnmounted, ref } from 'vue'
import { MiuixCard, MiuixText } from 'miuix-vue'
import AppIcon from '../components/AppIcon.vue'
import TweetCard from '../components/TweetCard.vue'
import { bookmarks, toXApiError } from '../api/x'
import type { XPost } from '../types'

const props = defineProps<{ loggedIn: boolean }>()
const emit = defineEmits<{
  open: [id: string]
  back: []
  like: [post: XPost]
  repost: [post: XPost]
  bookmark: [post: XPost]
  reply: [id: string]
  login: []
}>()

const posts = ref<XPost[]>([])
const cursor = ref<string | null>(null)
const loading = ref(false)
const loadingMore = ref(false)
const done = ref(false)
const error = ref('')
const loaded = ref(false)

const sentinel = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | null = null

async function load(reset = false): Promise<void> {
  if (!props.loggedIn) return
  if (reset) {
    posts.value = []
    cursor.value = null
    done.value = false
    loaded.value = false
  }
  if (done.value) return
  const first = !cursor.value
  if (first) loading.value = true
  else loadingMore.value = true
  error.value = ''

  try {
    const page = await bookmarks(cursor.value)
    posts.value = merge(posts.value, page.posts)
    cursor.value = page.bottomCursor ?? null
    done.value = !page.bottomCursor || page.posts.length === 0
    loaded.value = true
  } catch (e) {
    const err = toXApiError(e)
    error.value = err.kind === 'unauthorized' ? '需要登录 X 账号' : err.message
  } finally {
    loading.value = false
    loadingMore.value = false
  }
}

function merge(list: XPost[], more: XPost[]): XPost[] {
  const seen = new Set(list.map((p) => p.id))
  return [...list, ...more.filter((p) => !seen.has(p.id))]
}

onMounted(() => {
  void load()
  observer = new IntersectionObserver(
    (entries) => {
      if (entries.some((e) => e.isIntersecting) && !loading.value && !loadingMore.value) {
        void load()
      }
    },
    { rootMargin: '300px' },
  )
  if (sentinel.value) observer.observe(sentinel.value)
})

onUnmounted(() => observer?.disconnect())
</script>

<template>
  <div class="ck-bm">
    <header class="ck-bm__head">
      <button type="button" class="ck-bm__back" aria-label="返回" @click="emit('back')">
        <AppIcon name="arrowLeft" :size="20" />
      </button>
      <MiuixText type="title3" weight="medium" class="ck-bm__title">书签</MiuixText>
      <span class="ck-bm__spacer" />
    </header>

    <div class="ck-bm__scroll">
      <div class="ck-bm__content">
        <div v-if="!loggedIn" class="ck-empty">
          <div class="ck-empty__art">
            <AppIcon name="bookmark" :size="32" :stroke-width="1.4" />
          </div>
          <h2 class="ck-empty__title">还没有登录</h2>
          <p class="ck-empty__desc">登录 X 账号后，这里会显示你的书签。</p>
          <button type="button" class="ck-primary-btn" @click="emit('login')">登录 X</button>
        </div>

        <div v-else-if="loading" class="ck-bm__status">加载中…</div>

        <div v-else-if="error" class="ck-empty">
          <p class="ck-empty__desc">{{ error }}</p>
          <button type="button" class="ck-ghost-btn" @click="load(true)">重试</button>
        </div>

        <div v-else-if="loaded && !posts.length" class="ck-empty">
          <div class="ck-empty__art">
            <AppIcon name="bookmark" :size="32" :stroke-width="1.4" />
          </div>
          <h2 class="ck-empty__title">还没有书签</h2>
          <p class="ck-empty__desc">在帖子上点书签按钮即可保存。</p>
        </div>

        <template v-else>
          <TweetCard
            v-for="post in posts"
            :key="post.id"
            :post="post"
            @open="emit('open', $event)"
            @like="emit('like', $event)"
            @repost="emit('repost', $event)"
            @bookmark="emit('bookmark', $event)"
            @reply="emit('reply', $event)"
          />
          <div ref="sentinel" class="ck-bm__sentinel">
            <span v-if="loadingMore">正在加载更多…</span>
            <span v-else-if="done">已到达末尾</span>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ck-bm {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--ck-bg);
}

.ck-bm__head {
  position: relative;
  flex: none;
  justify-content: space-between;
  display: flex;
  align-items: center;
  height: 56px;
  padding: 0 8px;
}
.ck-bm__back {
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
.ck-bm__title {  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  max-width: 56%;
  text-align: center;
  pointer-events: none;
  color: var(--ck-text);
}
.ck-bm__spacer {
  width: 34px;
}

.ck-bm__scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  scrollbar-width: none;
}
.ck-bm__scroll::-webkit-scrollbar {
  width: 0;
}

.ck-bm__content {
  display: flex;
  flex-direction: column;
  gap: var(--ck-page-gap);
  padding: 0 10px 24px;
}


.ck-bm__status {
  padding: 30px 0;
  text-align: center;
  font-size: 12px;
  color: var(--ck-text-secondary);
}

.ck-bm__sentinel {
  padding: 14px 0 4px;
  text-align: center;
  font-size: 11px;
  color: var(--ck-text-tertiary);
}
</style>
