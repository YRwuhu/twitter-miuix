<script setup lang="ts">
/** 搜索页：真实帖子搜索 + 用户搜索。 */
import { computed, ref } from 'vue'
import { MiuixCard, MiuixSearchBar, MiuixTabRow, MiuixText } from 'miuix-vue'
import AppIcon from '../components/AppIcon.vue'
import AvatarBadge from '../components/AvatarBadge.vue'
import TweetCard from '../components/TweetCard.vue'
import VerifiedBadge from '../components/VerifiedBadge.vue'
import { searchTimeline, toXApiError, userByScreenName, xTrends } from '../api/x'
import { compactCount, type XPost, type XUser } from '../types'

const props = defineProps<{ loggedIn: boolean }>()
const emit = defineEmits<{
  open: [id: string]
  openUser: [user: XUser]
  like: [post: XPost]
  repost: [post: XPost]
  bookmark: [post: XPost]
  reply: [id: string]
  login: []
}>()

const query = ref('')
const mode = ref(0) // 0 = 帖子, 1 = 用户
const modeTabs = ['帖子', '用户', '趋势']

const loading = ref(false)
const error = ref('')
const posts = ref<XPost[]>([])
const user = ref<XUser | null>(null)
const searched = ref(false)
const trends = ref<{ name: string; query: string; tweetVolume: number | null }[]>([])
const trendsLoading = ref(false)

/** X 的 trends 响应是 [{ trends: [...] }]。 */
async function loadTrends(): Promise<void> {
  trendsLoading.value = true
  error.value = ''
  try {
    const raw = (await xTrends(1)) as unknown as { trends?: unknown[] }[]
    const list = (raw?.[0]?.trends ?? []) as {
      name?: string
      query?: string
      tweet_volume?: number | null
    }[]
    trends.value = list
      .filter((t) => !!t.name)
      .map((t) => ({
        name: t.name!,
        query: t.query ?? t.name!,
        tweetVolume: t.tweet_volume ?? null,
      }))
  } catch (e) {
    error.value = toXApiError(e).message
  } finally {
    trendsLoading.value = false
  }
}

const canSearch = computed(() => query.value.trim().length > 0 && !loading.value)

async function run(): Promise<void> {
  const q = query.value.trim()
  if (!q || loading.value) return
  loading.value = true
  error.value = ''
  posts.value = []
  user.value = null
  searched.value = true

  try {
    if (mode.value === 2) {
      await loadTrends()
      return
    }
    if (mode.value === 1) {
      // 用户搜索走 UserByScreenName，游客也可用
      user.value = await userByScreenName(q.replace(/^@/, ''))
    } else {
      if (!props.loggedIn) {
        error.value = '需要登录 X 账号才能搜索帖子'
        return
      }
      const page = await searchTimeline(q, 'Latest')
      posts.value = page.posts
    }
  } catch (e) {
    const err = toXApiError(e)
    if (err.kind === 'not_found') {
      error.value = mode.value === 1 ? `没有找到 @${q}` : '没有找到相关帖子'
    } else if (err.kind === 'unauthorized') {
      error.value = '需要登录 X 账号'
    } else {
      error.value = err.message
    }
  } finally {
    loading.value = false
  }
}

function onModeChange(value: number): void {
  mode.value = value
  if (value === 2) {
    void loadTrends()
  } else if (searched.value && query.value.trim()) {
    void run()
  }
}
</script>

<template>
  <div class="ck-search">
    <header class="ck-search__head">
      <h1 class="ck-page-title">搜索</h1>
    </header>

    <div class="ck-search__bar">
      <MiuixSearchBar v-model="query" label="搜索帖子或用户" @search="run" />
    </div>

    <div class="ck-search__tabs">
      <MiuixTabRow :model-value="mode" :tabs="modeTabs" @update:model-value="onModeChange" />
    </div>

    <div class="ck-search__scroll">
      <div class="ck-search__content">
        <div v-if="loading" class="ck-search__status">搜索中…</div>

        <div v-else-if="error" class="ck-empty">
          <p class="ck-empty__desc">{{ error }}</p>
          <button type="button" class="ck-ghost-btn" @click="mode === 2 ? loadTrends() : run()">
            重试
          </button>
        </div>

        <!-- 用户结果 -->
        <MiuixCard
          v-else-if="user"
          class="ck-user-card"
          press-feedback="none"
          show-indication
          @click="emit('openUser', user)"
        >
          <div class="ck-user__banner">
            <img
              v-if="user.banner"
              :src="user.banner"
              alt=""
              class="ck-user__banner-img"
              referrerpolicy="no-referrer"
            />
          </div>
          <div class="ck-user__identity">
            <AvatarBadge :name="user.name" :avatar="user.avatar" :size="64" />
          </div>
          <div class="ck-user__names">
            <div class="ck-user__name-row">
              <strong>{{ user.name }}</strong>
              <VerifiedBadge :verified="user.verified" :size="14" />
            </div>
            <span class="ck-user__handle">@{{ user.handle }}</span>
            <p v-if="user.bio" class="ck-user__bio">{{ user.bio }}</p>
          </div>
          <div class="ck-user__stats">
            <span><strong>{{ compactCount(user.tweets) || '0' }}</strong> 帖子</span>
            <span><strong>{{ compactCount(user.following) || '0' }}</strong> 正在关注</span>
            <span><strong>{{ compactCount(user.followers) || '0' }}</strong> 关注者</span>
          </div>
        </MiuixCard>

        <!-- 趋势 -->
        <template v-if="mode === 2">
          <div v-if="trendsLoading" class="ck-search__status">加载趋势…</div>
          <MiuixCard v-else-if="trends.length" class="ck-trends" press-feedback="none">
            <h3>正在发生的趋势</h3>
            <button
              v-for="(t, i) in trends"
              :key="t.name"
              type="button"
              class="ck-trends__item"
              @click="query = t.name.replace(/^#/, ''); mode = 0; run()"
            >
              <span class="ck-trends__rank">{{ i + 1 }}</span>
              <div class="ck-trends__body">
                <strong>{{ t.name }}</strong>
                <small v-if="t.tweetVolume">{{ t.tweetVolume.toLocaleString('zh-CN') }} 条帖子</small>
              </div>
            </button>
          </MiuixCard>
        </template>

        <!-- 帖子结果 -->
        <template v-else-if="posts.length">
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
        </template>

        <div v-else-if="searched" class="ck-empty">
          <p class="ck-empty__desc">没有找到结果</p>
        </div>

        <div v-else class="ck-empty">
          <div class="ck-empty__art">
            <AppIcon name="search" :size="32" :stroke-width="1.4" />
          </div>
          <h2 class="ck-empty__title">搜索</h2>
          <p class="ck-empty__desc">搜索帖子或用户。帖子搜索需要登录 X 账号。</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ck-search {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--ck-bg);
}

.ck-search__head {
  flex: none;
  padding: 10px 18px 6px;
}

.ck-search__bar {
  flex: none;
  padding: 2px 10px 8px;
}
.ck-search__bar :deep(.m-search-bar__field) {
  background: var(--ck-card);
}

.ck-search__tabs {
  flex: none;
  padding: 0 16px 10px;
}

.ck-search__scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  scrollbar-width: none;
}
.ck-search__scroll::-webkit-scrollbar {
  width: 0;
}

.ck-search__content {
  display: flex;
  flex-direction: column;
  gap: var(--ck-page-gap);
  padding: 0 10px 24px;
}

.ck-search__status {
  padding: 30px 0;
  text-align: center;
  font-size: 12px;
  color: var(--ck-text-secondary);
}


/* 用户卡片 */
.ck-user-card {
  --m-card-color: var(--ck-card);
  box-shadow: var(--ck-shadow-card);
  overflow: hidden;
}
.ck-user__banner {
  height: 84px;
  overflow: hidden;
  background: linear-gradient(120deg, #1d9bf0 0%, #7856ff 58%, #f91880 100%);
}
.ck-user__banner-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
.ck-user__identity {
  display: flex;
  padding: 0 14px;
  margin-top: -30px;
}
.ck-user__identity :deep(.ck-avatar) {
  border: 4px solid var(--ck-card);
  box-sizing: content-box;
}
.ck-user__names {
  padding: 8px 14px 0;
}
.ck-user__name-row {
  display: flex;
  align-items: center;
  gap: 5px;
}
.ck-user__name-row strong {
  font-size: 17px;
  font-weight: 800;
  color: var(--ck-text);
}
.ck-user__handle {
  display: block;
  margin-top: 1px;
  font-size: 12px;
  color: var(--ck-text-secondary);
}
.ck-user__bio {
  margin: 9px 0 0;
  font-size: 13px;
  line-height: 18px;
  color: var(--ck-text);
  white-space: pre-wrap;
  word-break: break-word;
}
.ck-user__stats {
  display: flex;
  gap: 16px;
  padding: 12px 14px 16px;
  font-size: 12px;
  color: var(--ck-text-secondary);
}
.ck-user__stats strong {
  color: var(--ck-text);
  font-weight: 700;
}
</style>
/* 趋势列表 */
.ck-trends {
  --m-card-color: var(--ck-card);
  box-shadow: var(--ck-shadow-card);
  padding: 8px 0;
}
.ck-trends h3 {
  margin: 0;
  padding: 8px 14px 10px;
  font-size: 15px;
  font-weight: 800;
  color: var(--ck-text);
}
.ck-trends__item {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 9px 14px;
  border: 0;
  border-top: 1px solid var(--ck-border);
  background: transparent;
  text-align: left;
  cursor: pointer;
}
.ck-trends__item:hover {
  background: var(--ck-bg-sunken);
}
.ck-trends__rank {
  width: 22px;
  flex: none;
  font-size: 12px;
  font-weight: 700;
  color: var(--ck-text-tertiary);
  font-variant-numeric: tabular-nums;
}
.ck-trends__body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.ck-trends__body strong {
  font-size: 13px;
  color: var(--ck-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.ck-trends__body small {
  font-size: 11px;
  color: var(--ck-text-secondary);
}
