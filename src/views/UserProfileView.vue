<script setup lang="ts">
/** 用户主页：真实资料 + 帖子/回复/媒体/喜欢。 */
import { computed, ref, watch } from 'vue'
import { MiuixCard, MiuixTabRow, MiuixText } from 'miuix-vue'
import AppIcon from '../components/AppIcon.vue'
import AvatarBadge from '../components/AvatarBadge.vue'
import TweetCard from '../components/TweetCard.vue'
import VerifiedBadge from '../components/VerifiedBadge.vue'
import { follow as apiFollow, toXApiError, userTimeline } from '../api/x'
import { compactCount, type UserTimelineKind, type XPost, type XUser } from '../types'

const props = defineProps<{ user: XUser | null }>()
const emit = defineEmits<{
  back: []
  open: [id: string]
  like: [post: XPost]
  repost: [post: XPost]
  bookmark: [post: XPost]
  reply: [id: string]
}>()

const tabs = ['帖子', '回复', '媒体', '喜欢']
const tabKeys: UserTimelineKind[] = ['posts', 'replies', 'media', 'likes']
const tabIndex = ref(0)

const posts = ref<XPost[]>([])
const cursor = ref<string | null>(null)
const loading = ref(false)
const loadingMore = ref(false)
const done = ref(false)
const error = ref('')
const following = ref(false)
const followBusy = ref(false)
const followMsg = ref('')

async function toggleFollow(): Promise<void> {
  const id = props.user?.id
  if (!id || followBusy.value) return
  followBusy.value = true
  followMsg.value = ''
  const next = !following.value
  try {
    await apiFollow(id, next)
    following.value = next
    followMsg.value = next ? '已关注' : '已取消关注'
  } catch (e) {
    followMsg.value = toXApiError(e).message
  } finally {
    followBusy.value = false
  }
}

const stats = computed(() => {
  const u = props.user
  if (!u) return []
  return [
    { label: '帖子', value: compactCount(u.tweets) || '0' },
    { label: '正在关注', value: compactCount(u.following) || '0' },
    { label: '关注者', value: compactCount(u.followers) || '0' },
  ]
})

async function load(reset = false): Promise<void> {
  const user = props.user
  if (!user?.id) return
  if (reset) {
    posts.value = []
    cursor.value = null
    done.value = false
  }
  if (done.value && !reset) return

  const first = !cursor.value
  if (first) loading.value = true
  else loadingMore.value = true
  error.value = ''

  try {
    const page = await userTimeline(user.id, tabKeys[tabIndex.value], cursor.value)
    posts.value = reset || first ? page.posts : merge(posts.value, page.posts)
    cursor.value = page.bottomCursor ?? null
    done.value = !page.bottomCursor || page.posts.length === 0
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

function onTabChange(value: number): void {
  tabIndex.value = value
  void load(true)
}

watch(
  () => props.user?.id,
  () => {
    tabIndex.value = 0
    following.value = props.user?.isFollowing ?? false
    void load(true)
  },
  { immediate: true },
)
</script>

<template>
  <div class="ck-up">
    <header class="ck-up__head">
      <button type="button" class="ck-up__back" aria-label="返回" @click="emit('back')">
        <AppIcon name="arrowLeft" :size="20" />
      </button>
      <MiuixText type="title3" weight="medium" class="ck-up__title">
        {{ user?.name || '用户' }}
      </MiuixText>
      <span class="ck-up__spacer" />
    </header>

    <div class="ck-up__scroll">
      <div class="ck-up__content">
        <MiuixCard v-if="user" class="ck-up__profile" press-feedback="none">
          <div class="ck-up__banner">
            <img
              v-if="user.banner"
              :src="user.banner"
              alt=""
              class="ck-up__banner-img"
              referrerpolicy="no-referrer"
            />
          </div>
          <div class="ck-up__identity">
            <AvatarBadge :name="user.name" :avatar="user.avatar" :size="68" />
          </div>
          <div class="ck-up__names">
            <div class="ck-up__name-row">
              <strong>{{ user.name }}</strong>
              <VerifiedBadge :verified="user.verified" :size="14" />
            </div>
            <span class="ck-up__handle">@{{ user.handle }}</span>
            <p v-if="user.bio" class="ck-up__bio">{{ user.bio }}</p>
          </div>
          <div class="ck-up__stats">
            <span v-for="stat in stats" :key="stat.label">
              <strong>{{ stat.value }}</strong> {{ stat.label }}
            </span>
          </div>
          <div class="ck-up__actions">
            <button
              type="button"
              class="ck-up__follow"
              :class="{ 'is-following': following }"
              :disabled="followBusy"
              @click="toggleFollow"
            >
              {{ following ? '正在关注' : '关注' }}
            </button>
            <span v-if="followMsg" class="ck-up__follow-msg">{{ followMsg }}</span>
          </div>
        </MiuixCard>

        <div class="ck-up__tabs">
          <MiuixTabRow :model-value="tabIndex" :tabs="tabs" @update:model-value="onTabChange" />
        </div>

        <div v-if="loading" class="ck-up__status">加载中…</div>

        <div v-else-if="error" class="ck-empty">
          <p class="ck-empty__desc">{{ error }}</p>
          <button type="button" class="ck-ghost-btn" @click="load(true)">重试</button>
        </div>

        <div v-else-if="!posts.length" class="ck-empty">
          <p class="ck-empty__desc">这里还没有内容</p>
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
          <div class="ck-up__sentinel">
            <span v-if="loadingMore">正在加载更多…</span>
            <span v-else-if="done">已到达末尾</span>
            <button v-else type="button" class="ck-up__more" @click="load()">加载更多</button>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ck-up {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--ck-bg);
}

.ck-up__head {
  position: relative;
  flex: none;
  justify-content: space-between;
  display: flex;
  align-items: center;
  height: 56px;
  padding: 0 8px;
}
.ck-up__back {
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
.ck-up__title {  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  max-width: 56%;
  text-align: center;
  pointer-events: none;
  color: var(--ck-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.ck-up__spacer {
  width: 34px;
}

.ck-up__scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  scrollbar-width: none;
}
.ck-up__scroll::-webkit-scrollbar {
  width: 0;
}

.ck-up__content {
  display: flex;
  flex-direction: column;
  gap: var(--ck-page-gap);
  padding: 0 10px 24px;
}

.ck-up__profile {
  --m-card-color: var(--ck-card);
  box-shadow: var(--ck-shadow-card);
  overflow: hidden;
}

.ck-up__banner {
  height: 90px;
  overflow: hidden;
  background: linear-gradient(120deg, #1d9bf0 0%, #7856ff 58%, #f91880 100%);
}
.ck-up__banner-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
.ck-up__identity {
  display: flex;
  padding: 0 14px;
  margin-top: -32px;
}
.ck-up__identity :deep(.ck-avatar) {
  border: 4px solid var(--ck-card);
  box-sizing: content-box;
}
.ck-up__names {
  padding: 8px 14px 0;
}
.ck-up__name-row {
  display: flex;
  align-items: center;
  gap: 5px;
}
.ck-up__name-row strong {
  font-size: 18px;
  font-weight: 800;
  color: var(--ck-text);
}
.ck-up__handle {
  display: block;
  margin-top: 1px;
  font-size: 13px;
  color: var(--ck-text-secondary);
}
.ck-up__bio {
  margin: 9px 0 0;
  font-size: 13px;
  line-height: 18px;
  color: var(--ck-text);
  white-space: pre-wrap;
  word-break: break-word;
}
.ck-up__stats {
  display: flex;
  gap: 16px;
  padding: 12px 14px 14px;
  font-size: 12px;
  color: var(--ck-text-secondary);
}
.ck-up__stats strong {
  color: var(--ck-text);
  font-weight: 700;
}

.ck-up__tabs {
  padding: 0 6px;
}

.ck-up__status {
  padding: 30px 0;
  text-align: center;
  font-size: 12px;
  color: var(--ck-text-secondary);
}


.ck-up__sentinel {
  padding: 14px 0 4px;
  text-align: center;
  font-size: 11px;
  color: var(--ck-text-tertiary);
}
.ck-up__more {
  border: 0;
  background: transparent;
  color: var(--ck-blue);
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}
</style>
.ck-up__actions {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 14px 14px;
}
.ck-up__follow {
  height: 32px;
  padding: 0 20px;
  border: 0;
  border-radius: 9999px;
  background: var(--ck-blue);
  color: #fff;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}
.ck-up__follow.is-following {
  background: var(--ck-bg-sunken);
  color: var(--ck-text);
}
.ck-up__follow:disabled {
  opacity: 0.6;
}
.ck-up__follow-msg {
  font-size: 11px;
  color: var(--ck-text-secondary);
}
