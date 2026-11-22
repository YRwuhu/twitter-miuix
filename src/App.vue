<script setup lang="ts">
/**
 * 应用外壳：页面编排、登录态、互动。
 *
 * 数据全部来自 Rust 侧真实接口；未登录时首页显示登录引导。
 */
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { MiuixSwitch, MiuixText } from 'miuix-vue'
import BottomNav from './components/BottomNav.vue'
import NotificationsView from './views/NotificationsView.vue'
import BookmarksView from './views/BookmarksView.vue'
import UserProfileView from './views/UserProfileView.vue'
import ComposeSheet from './components/ComposeSheet.vue'
import LoginSheet from './components/LoginSheet.vue'
import DetailView from './views/DetailView.vue'
import ExploreView from './views/ExploreView.vue'
import HomeView from './views/HomeView.vue'
import MessagesView from './views/MessagesView.vue'
import ProfileView from './views/ProfileView.vue'
import { authStatus, restoreSession, favorite, retweet, bookmark as apiBookmark, createTweet, logout as apiLogout, setQueryId, toXApiError, tweetDetail } from './api/x'
import type { AuthStatus } from './api/x'
import { useTimelineStore } from './stores/timeline'
import { applyTheme, theme } from './theme'
import type { TabKey, ThemeChoice, XPost, XUser } from './types'

const tab = ref<TabKey>('home')
const detailId = ref<string | null>(null)
/** 二级页面：书签 / 他人主页 */
const overlay = ref<'bookmarks' | 'user' | null>(null)
const overlayUser = ref<XUser | null>(null)
const composeOpen = ref(false)
const replyTarget = ref<{ id: string; handle: string } | null>(null)
const settingsOpen = ref(false)
const loginOpen = ref(false)
const toast = ref<string | null>(null)
let toastTimer: ReturnType<typeof setTimeout> | null = null

const auth = ref<AuthStatus>({ logged_in: false, guest: false, user: null })
const me = ref<XUser | null>(null)
const timeline = useTimelineStore()

/* 设置项 */
const reduceMotion = ref(readReduceMotion())
const queryIdInput = ref('')
const queryIdTarget = ref('HomeTimeline')
const queryIdBusy = ref(false)

function readReduceMotion(): boolean {
  try {
    return localStorage.getItem('twitter-miuix.reduce-motion') === 'true'
  } catch {
    return false
  }
}

const loggedIn = computed(() => auth.value.logged_in)
const showNav = computed(() => !detailId.value)

/** 详情页需要拿到完整的帖子对象：从时间线缓存里找。 */
const detailReplies = ref<XPost[]>([])
const detailLoading = ref(false)

/** 详情主帖：优先用接口返回的，其次从时间线缓存找。 */
const activePost = computed<XPost | null>(() => {
  if (!detailId.value) return null
  const all = [...timeline.forYou.posts, ...timeline.following.posts]
  return detailMain.value ?? all.find((p) => p.id === detailId.value) ?? null
})
const detailMain = ref<XPost | null>(null)
let detailRequest = 0

/** 打开详情并拉取真实回复。 */
async function loadDetail(id: string): Promise<void> {
  const request = ++detailRequest
  detailMain.value = null
  detailReplies.value = []
  detailLoading.value = true
  try {
    const page = await tweetDetail(id)
    if (request !== detailRequest || detailId.value !== id) return
    if (page.posts.length) {
      detailMain.value = page.posts[0]
      detailReplies.value = page.posts.slice(1)
    }
  } catch (error) {
    if (request === detailRequest) notify(toXApiError(error).message)
  } finally {
    if (request === detailRequest) detailLoading.value = false
  }
}

function notify(message: string): void {
  toast.value = message
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => (toast.value = null), 2400)
}

async function refreshAuth(): Promise<void> {
  try {
    // 先尝试恢复上次登录态（DPAPI），失败再退回普通状态查询
    auth.value = await restoreSession()
    if (!auth.value.logged_in) auth.value = await authStatus()
    me.value = auth.value.user ?? null
    if (auth.value.logged_in) {
      await timeline.load(timeline.active, true)
    }
  } catch {
    // 浏览器预览没有 Tauri 命令
  }
}

function onLoginSuccess(status: AuthStatus): void {
  auth.value = status
  me.value = status.user ?? null
  notify(status.logged_in ? '已登录 X' : '已切换到游客身份')
  if (status.logged_in) {
    void timeline.load(timeline.active, true)
  }
}

async function doLogout(): Promise<void> {
  try {
    auth.value = await apiLogout()
    me.value = null
    timeline.reset()
    notify('已退出登录')
  } catch (error) {
    notify(toXApiError(error).message)
  }
}

/* ---------------- 互动 ---------------- */

/**
 * 互动走真实 GraphQL。乐观更新先落地，失败再回滚。
 */
async function toggleLike(id: string): Promise<void> {
  const post = findPost(id)
  if (!post) return
  const target = post.reposted ?? post
  const next = !target.isLiked

  timeline.patchPost(id, {
    isLiked: next,
    metrics: { ...target.metrics, like: target.metrics.like + (next ? 1 : -1) },
  })

  try {
    await favorite(target.id, next)
  } catch (error) {
    // 回滚
    timeline.patchPost(id, {
      isLiked: !next,
      metrics: { ...target.metrics, like: target.metrics.like },
    })
    notify(toXApiError(error).message)
  }
}

async function toggleRepost(id: string): Promise<void> {
  const post = findPost(id)
  if (!post) return
  const target = post.reposted ?? post
  const next = !target.isReposted

  timeline.patchPost(id, {
    isReposted: next,
    metrics: { ...target.metrics, repost: target.metrics.repost + (next ? 1 : -1) },
  })

  try {
    await retweet(target.id, next)
    notify(next ? '已转推' : '已撤销转推')
  } catch (error) {
    timeline.patchPost(id, {
      isReposted: !next,
      metrics: { ...target.metrics, repost: target.metrics.repost },
    })
    notify(toXApiError(error).message)
  }
}

async function toggleBookmark(id: string): Promise<void> {
  const post = findPost(id)
  if (!post) return
  const target = post.reposted ?? post
  const next = !target.isBookmarked

  timeline.patchPost(id, { isBookmarked: next })

  try {
    await apiBookmark(target.id, next)
    notify(next ? '已加入书签' : '已从书签移除')
  } catch (error) {
    timeline.patchPost(id, { isBookmarked: !next })
    notify(toXApiError(error).message)
  }
}

function findPost(id: string): XPost | null {
  const all = [...timeline.forYou.posts, ...timeline.following.posts]
  return all.find((p) => p.id === id) ?? null
}

/* ---------------- 导航 ---------------- */

function openDetail(id: string): void {
  detailId.value = id
  void loadDetail(id)
}

function openCompose(replyToId?: string): void {
  if (!loggedIn.value) {
    loginOpen.value = true
    return
  }
  if (replyToId) {
    const post = findPost(replyToId)
    const target = post?.reposted ?? post
    replyTarget.value = { id: replyToId, handle: target?.author.handle ?? '' }
  } else {
    replyTarget.value = null
  }
  composeOpen.value = true
}

async function publish(text: string): Promise<void> {
  try {
    await createTweet(text, replyTarget.value?.id ?? null)
    notify(replyTarget.value ? '回复已发送' : '帖子已发布')
    replyTarget.value = null
    // 发布后刷新时间线，让新帖出现在正确位置
    await timeline.refresh('following')
    tab.value = 'home'
  } catch (error) {
    notify(toXApiError(error).message)
  }
}

function changeTab(next: TabKey): void {
  detailRequest++
  tab.value = next
  detailId.value = null
  overlay.value = null
}

function openBookmarks(): void {
  overlay.value = 'bookmarks'
  detailId.value = null
}

function openUser(user: XUser): void {
  overlayUser.value = user
  overlay.value = 'user'
  detailId.value = null
}

function closeOverlay(): void {
  overlay.value = null
  overlayUser.value = null
}

function selectTheme(next: ThemeChoice): void {
  applyTheme(next)
}

watch(reduceMotion, (enabled) => {
  document.documentElement.classList.toggle('ck-reduce-motion', enabled)
  try {
    localStorage.setItem('twitter-miuix.reduce-motion', String(enabled))
  } catch {
    // Keep the preference for this session if storage is unavailable.
  }
}, { immediate: true })

async function saveQueryId(): Promise<void> {
  const operation = queryIdTarget.value.trim()
  const id = queryIdInput.value.trim()
  if (!operation || !id) return
  queryIdBusy.value = true
  try {
    await setQueryId(operation, id)
    notify(`已更新 ${operation} 的 queryId`)
    queryIdInput.value = ''
  } catch (error) {
    notify(toXApiError(error).message)
  } finally {
    queryIdBusy.value = false
  }
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key !== 'Escape') return
  if (composeOpen.value) {
    composeOpen.value = false
    return
  }
  if (loginOpen.value) {
    loginOpen.value = false
    return
  }
  if (settingsOpen.value) {
    settingsOpen.value = false
    return
  }
  if (detailId.value) {
    detailId.value = null
    return
  }
  if (overlay.value) closeOverlay()
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
  void refreshAuth()
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown)
})

watch(tab, () => {
  detailId.value = null
})
</script>

<template>
  <div class="ck-shell">
    <main class="ck-viewport">
        <BookmarksView
          v-if="overlay === 'bookmarks'"
          key="bookmarks"
          :logged-in="loggedIn"
          @open="openDetail"
          @like="toggleLike"
          @repost="toggleRepost"
          @bookmark="toggleBookmark"
          @reply="openCompose"
          @login="loginOpen = true"
        />

        <UserProfileView
          v-else-if="overlay === 'user'"
          key="user"
          :user="overlayUser"
          @back="closeOverlay"
          @open="openDetail"
          @like="toggleLike"
          @repost="toggleRepost"
          @bookmark="toggleBookmark"
          @reply="openCompose"
        />

        <DetailView
          v-else-if="detailId"
          :key="`detail-${detailId}`"
          :post="activePost"
          :replies="detailReplies"
          :loading="detailLoading"
          @open="openDetail"
          @back="detailId = null"
          @like="toggleLike"
          @repost="toggleRepost"
          @bookmark="toggleBookmark"
          @reply="openCompose"
        />

        <HomeView
          v-else-if="tab === 'home'"
          key="home"
          :me="me"
          :logged-in="loggedIn"
          @like="toggleLike"
          @repost="toggleRepost"
          @bookmark="toggleBookmark"
          @reply="openCompose"
          @open="openDetail"
          @compose="openCompose()"
          @settings="settingsOpen = true"
          @notifications="tab = 'notifications'"
          @login="loginOpen = true"
        />

        <ExploreView
          v-else-if="tab === 'search'"
          key="search"
          :logged-in="loggedIn"
          @open="openDetail"
          @like="toggleLike"
          @repost="toggleRepost"
          @bookmark="toggleBookmark"
          @reply="openCompose"
          @login="loginOpen = true"
        />

        <NotificationsView
          v-else-if="tab === 'notifications'"
          key="notifications"
          :me="me"
          :logged-in="loggedIn"
          @open="openDetail"
          @login="loginOpen = true"
        />

        <MessagesView
          v-else-if="tab === 'messages'"
          key="messages"
          :me="me"
          :logged-in="loggedIn"
          @login="loginOpen = true"
          @open-user="openUser"
        />

        <ProfileView
          v-else
          key="me"
          :me="me"
          :logged-in="loggedIn"
          :theme="theme"
          @theme="selectTheme"
          @settings="settingsOpen = true"
          @login="loginOpen = true"
          @logout="doLogout"
        />

      <BottomNav v-if="showNav" :active="tab" :unread="0" @change="changeTab" />
    </main>

    <LoginSheet v-model="loginOpen" @success="onLoginSuccess" />

    <ComposeSheet
      v-model="composeOpen"
      :reply-to="replyTarget ? `@${replyTarget.handle}` : undefined"
      @publish="publish"
    />

    <!-- 设置面板 -->
    <Transition name="ck-sheet">
      <div v-if="settingsOpen" class="ck-settings">
        <div class="ck-settings__scrim" @click="settingsOpen = false" />
        <div class="ck-settings__sheet" role="dialog" aria-modal="true" aria-label="设置">
          <header class="ck-settings__head">
            <h2>设置</h2>
            <button type="button" class="ck-settings__close" aria-label="关闭" @click="settingsOpen = false">
              ✕
            </button>
          </header>

          <div class="ck-settings__body">
            <h3>账号</h3>
            <div class="ck-settings__row">
              <span>{{ loggedIn ? '已登录 X' : auth.guest ? '游客身份' : '未登录' }}</span>
              <div class="ck-settings__inline">
                <button v-if="!loggedIn" type="button" class="ck-btn" @click="loginOpen = true">登录</button>
                <button v-else type="button" class="ck-btn" @click="doLogout">退出</button>
              </div>
            </div>

            <h3>内容</h3>
            <div class="ck-settings__row">
              <span>我的书签</span>
              <div class="ck-settings__inline">
                <button type="button" class="ck-btn" @click="settingsOpen = false; openBookmarks()">
                  打开
                </button>
              </div>
            </div>

            <h3>外观</h3>
            <div class="ck-settings__row">
              <span>主题</span>
              <div class="ck-settings__options">
                <button
                  v-for="option in [
                    { key: 'light', label: '浅色' },
                    { key: 'dark', label: '暗色' },
                    { key: 'black', label: '纯黑' },
                  ]"
                  :key="option.key"
                  type="button"
                  :class="{ 'is-selected': theme === option.key }"
                  @click="selectTheme(option.key as ThemeChoice)"
                >
                  {{ option.label }}
                </button>
              </div>
            </div>
            <div class="ck-settings__row">
              <span>减少动态效果</span>
              <MiuixSwitch v-model="reduceMotion" aria-label="减少动态效果" />
            </div>

            <h3>接口</h3>
            <div class="ck-settings__row ck-settings__row--stack">
              <span class="ck-settings__hint">
                当某个接口返回 404 时，说明 queryId 已被 X 轮换，在这里填入新的值。
              </span>
              <input
                v-model="queryIdTarget"
                class="ck-input"
                placeholder="操作名，例如 HomeTimeline"
                spellcheck="false"
              />
              <input
                v-model="queryIdInput"
                class="ck-input"
                placeholder="新的 queryId"
                spellcheck="false"
              />
              <button
                type="button"
                class="ck-btn ck-btn--primary"
                :disabled="queryIdBusy || !queryIdTarget.trim() || !queryIdInput.trim()"
                @click="saveQueryId"
              >
                保存
              </button>
            </div>

            <p class="ck-settings__note">
              登录凭据由 Windows DPAPI 加密保存。请求均为真实 X GraphQL 调用。
            </p>
          </div>
        </div>
      </div>
    </Transition>

    <Transition name="ck-toast">
      <div v-if="toast" class="ck-toast">{{ toast }}</div>
    </Transition>
  </div>
</template>

<style scoped>
.ck-shell {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  background: var(--ck-bg);
  overflow: hidden;
}

.ck-viewport {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}


/* 设置面板 */
.ck-settings {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  align-items: flex-end;
}
.ck-settings__scrim {
  position: absolute;
  inset: 0;
  background: var(--ck-overlay);
}
.ck-settings__sheet {
  position: relative;
  width: 100%;
  max-height: 84%;
  display: flex;
  flex-direction: column;
  border-radius: 26px 26px 0 0;
  background: var(--ck-card);
  box-shadow: 0 -8px 40px rgba(0, 0, 0, 0.24);
}
.ck-settings__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 16px 10px;
  border-bottom: 1px solid var(--ck-border);
}
.ck-settings__head h2 {
  margin: 0;
  font-size: 17px;
  font-weight: 800;
  color: var(--ck-text);
}
.ck-settings__close {
  width: 30px;
  height: 30px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 9999px;
  background: transparent;
  color: var(--ck-text);
  font-size: 14px;
  cursor: pointer;
}
.ck-settings__close:hover {
  background: var(--ck-hover);
}

.ck-settings__body {
  padding: 10px 16px 22px;
  overflow-y: auto;
}
.ck-settings__body h3 {
  margin: 14px 0 8px;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--ck-text-secondary);
}

.ck-settings__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 0;
  border-bottom: 1px solid var(--ck-border);
  font-size: 14px;
  color: var(--ck-text);
}
.ck-settings__row--stack {
  flex-direction: column;
  align-items: stretch;
  gap: 8px;
}
.ck-settings__inline {
  display: flex;
  gap: 6px;
}
.ck-settings__hint {
  font-size: 11px;
  line-height: 16px;
  color: var(--ck-text-secondary);
}

.ck-settings__options {
  display: flex;
  gap: 4px;
  padding: 3px;
  border-radius: 9999px;
  background: var(--ck-bg-sunken);
}
.ck-settings__options button {
  height: 28px;
  padding: 0 12px;
  border: 0;
  border-radius: 9999px;
  background: transparent;
  color: var(--ck-text-secondary);
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}
.ck-settings__options button.is-selected {
  background: var(--ck-card);
  color: var(--ck-text);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.14);
}

.ck-btn {
  height: 30px;
  padding: 0 14px;
  border: 0;
  border-radius: 9999px;
  background: var(--ck-bg-sunken);
  color: var(--ck-text);
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}
.ck-btn:hover:not(:disabled) {
  background: var(--ck-border);
}
.ck-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.ck-btn--primary {
  background: var(--ck-blue);
  color: #fff;
}
.ck-btn--primary:hover:not(:disabled) {
  background: var(--ck-blue-hover);
}

.ck-input {
  width: 100%;
  height: 32px;
  padding: 0 10px;
  border: 1px solid var(--ck-border);
  border-radius: 10px;
  background: var(--ck-bg-sunken);
  color: var(--ck-text);
  font-size: 12px;
  font-family: ui-monospace, monospace;
  outline: none;
}
.ck-input:focus {
  border-color: var(--ck-blue);
}

.ck-settings__note {
  margin: 14px 0 0;
  font-size: 11px;
  line-height: 17px;
  color: var(--ck-text-secondary);
}

.ck-toast {
  position: fixed;
  left: 50%;
  bottom: 78px;
  transform: translateX(-50%);
  z-index: 1200;
  max-width: 300px;
  padding: 9px 14px;
  border-radius: 9999px;
  background: var(--ck-text);
  color: var(--ck-bg);
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.24);
  pointer-events: none;
}
.ck-toast-enter-active,
.ck-toast-leave-active {
  transition: opacity 160ms ease, transform 160ms ease;
}
.ck-toast-enter-from,
.ck-toast-leave-to {
  opacity: 0;
  transform: translate(-50%, 6px);
}

.ck-sheet-enter-active,
.ck-sheet-leave-active {
  transition: opacity 180ms ease;
}
.ck-sheet-enter-active .ck-settings__sheet,
.ck-sheet-leave-active .ck-settings__sheet {
  transition: transform 240ms cubic-bezier(0.32, 0.72, 0, 1);
}
.ck-sheet-enter-from,
.ck-sheet-leave-to {
  opacity: 0;
}
.ck-sheet-enter-from .ck-settings__sheet,
.ck-sheet-leave-to .ck-settings__sheet {
  transform: translateY(100%);
}
</style>
