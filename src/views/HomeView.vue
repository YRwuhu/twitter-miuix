<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { MiuixIconButton, MiuixTabRow, MiuixText } from 'miuix-vue'
import AppIcon from '../components/AppIcon.vue'
import AvatarBadge from '../components/AvatarBadge.vue'
import TweetCard from '../components/TweetCard.vue'
import { useTimelineStore } from '../stores/timeline'
import type { XPost, XUser } from '../types'

const props = defineProps<{ me?: XUser | null; loggedIn: boolean }>()
const emit = defineEmits<{
  like: [post: XPost]
  repost: [post: XPost]
  bookmark: [post: XPost]
  reply: [id: string]
  open: [id: string]
  compose: []
  settings: []
  notifications: []
  login: []
}>()

const timeline = useTimelineStore()
const tabs = ['为你推荐', '正在关注']
const tabIndex = computed({
  get: () => (timeline.active === 'forYou' ? 0 : 1),
  set: (value: number) => {
    void timeline.switchTo(value === 0 ? 'forYou' : 'following')
  },
})

const scroller = ref<HTMLElement | null>(null)
const current = computed(() => timeline.state)
const sentinel = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | null = null

function onTabChange(): void {
  // 切换标签后回到顶部
  scroller.value?.scrollTo({ top: 0 })
}

watch(() => timeline.active, onTabChange)

onMounted(() => {
  if (props.loggedIn) {
    void timeline.load(timeline.active)
  }
  // 滚到底部自动加载下一页
  observer = new IntersectionObserver(
    (entries) => {
      if (entries.some((e) => e.isIntersecting)) {
        void timeline.loadMore(timeline.active)
      }
    },
    { root: scroller.value, rootMargin: '300px' },
  )
  if (sentinel.value) observer.observe(sentinel.value)
})

onUnmounted(() => observer?.disconnect())

watch(
  () => props.loggedIn,
  (loggedIn) => {
    if (loggedIn) void timeline.load(timeline.active, true)
  },
)

function scrollToTop(): void {
  scroller.value?.scrollTo({ top: 0, behavior: 'smooth' })
}
</script>

<template>
  <div class="ck-home">
    <header class="ck-topbar">
      <div class="ck-topbar__leading">
        <AvatarBadge
          v-if="me"
          :name="me.name"
          :avatar="me.avatar"
          :size="30"
          @click="emit('settings')"
        />
        <AvatarBadge v-else name="?" :size="30" @click="emit('login')" />
      </div>
      <span class="ck-topbar__title">主页</span>
      <div class="ck-topbar__actions">
        <MiuixIconButton aria-label="通知" @click="emit('notifications')">
          <AppIcon name="bell" :size="21" />
        </MiuixIconButton>
        <MiuixIconButton aria-label="设置" @click="emit('settings')">
          <AppIcon name="settings" :size="21" />
        </MiuixIconButton>
      </div>
    </header>

    <div class="ck-home__tabs">
      <MiuixTabRow v-model="tabIndex" :tabs="tabs" />
    </div>

    <div ref="scroller" class="ck-home__scroll" @dblclick="scrollToTop">
      <div class="ck-home__content">
        <!-- 未登录 -->
        <div v-if="!loggedIn" class="ck-empty">
          <div class="ck-empty__art">
            <AppIcon name="home" :size="34" :stroke-width="1.4" />
          </div>
          <h2 class="ck-empty__title">还没有登录</h2>
          <p class="ck-empty__desc">
            登录 X 账号后，这里会显示你的时间线。
          </p>
          <button type="button" class="ck-primary-btn" @click="emit('login')">登录 X</button>
        </div>

        <template v-else>
          <!-- 首屏加载 -->
          <div v-if="current.loading && !current.posts.length" class="ck-home__loading">
            <div v-for="i in 4" :key="i" class="ck-skel">
              <span class="ck-skel__avatar" />
              <div class="ck-skel__lines">
                <span class="ck-skel__line ck-skel__line--short" />
                <span class="ck-skel__line" />
                <span class="ck-skel__line" />
              </div>
            </div>
          </div>

          <!-- 错误 -->
          <div v-else-if="current.error" class="ck-empty">
            <p class="ck-empty__desc">{{ current.error }}</p>
            <button type="button" class="ck-ghost-btn" @click="timeline.refresh(timeline.active)">
              重试
            </button>
          </div>

          <!-- 空 -->
          <div v-else-if="!current.posts.length" class="ck-empty">
            <p class="ck-empty__desc">这里还没有内容</p>
            <button type="button" class="ck-ghost-btn" @click="timeline.refresh(timeline.active)">
              刷新
            </button>
          </div>

          <!-- 帖子列表 -->
          <template v-else>
            <TweetCard
              v-for="post in current.posts"
              :key="post.id"
              :post="post"
              @like="emit('like', $event)"
              @repost="emit('repost', $event)"
              @bookmark="emit('bookmark', $event)"
              @reply="emit('reply', $event)"
              @open="emit('open', $event)"
            />

            <div ref="sentinel" class="ck-home__sentinel">
              <span v-if="current.loadingMore">正在加载更多…</span>
              <span v-else-if="current.done">已到达末尾</span>
              <span v-else-if="current.error">{{ current.error }}</span>
            </div>
          </template>
        </template>
      </div>
    </div>

    <button type="button" class="ck-fab" aria-label="发帖" @click="emit('compose')">
      <AppIcon name="edit" :size="23" :stroke-width="2.1" />
    </button>
  </div>
</template>

<style scoped>
.ck-home {
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--ck-bg);
}

/* ---------- 顶栏：HyperOS 大标题 ---------- */
.ck-topbar {
  position: relative;
  flex: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 60px;
  padding: 0 10px 0 16px;
  background: var(--ck-bg);
}

.ck-topbar__leading {
  width: 40px;
  flex: none;
  display: flex;
  align-items: center;
}

.ck-topbar__title {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  font-size: var(--ck-title-size);
  font-weight: var(--ck-title-weight);
  letter-spacing: -0.02em;
  color: var(--ck-text);
  pointer-events: none;
  white-space: nowrap;
}

.ck-topbar__actions {
  display: flex;
  align-items: center;
  gap: 2px;
  color: var(--ck-text);
}

/* ---------- 标签栏 ---------- */
.ck-home__tabs {
  flex: none;
  padding: 0 16px 8px;
}

/* ---------- 滚动区：平面列表，无卡片嵌套 ---------- */
.ck-home__scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  scrollbar-width: none;
}
.ck-home__scroll::-webkit-scrollbar {
  width: 0;
}

.ck-home__content {
  display: flex;
  flex-direction: column;
  padding: 0 0 100px;
}

/* ---------- 空状态：居中留白，不用卡片 ---------- */
.ck-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 72px 40px 40px;
  text-align: center;
}

.ck-empty__art {
  width: 76px;
  height: 76px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 6px;
  border-radius: 26px;
  background: var(--ck-bg-sunken);
  color: var(--ck-text-tertiary);
}

.ck-empty__title {
  margin: 0;
  font-size: 19px;
  font-weight: 700;
  letter-spacing: -0.01em;
  color: var(--ck-text);
}

.ck-empty__desc {
  margin: 0;
  max-width: 260px;
  font-size: 13px;
  line-height: 20px;
  color: var(--ck-text-secondary);
}

.ck-primary-btn {
  margin-top: 12px;
  height: 40px;
  padding: 0 28px;
  border: 0;
  border-radius: 9999px;
  background: var(--ck-blue);
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: transform 160ms var(--ck-ease-spring), background-color 160ms linear;
}
.ck-primary-btn:hover {
  background: var(--ck-blue-hover);
}
.ck-primary-btn:active {
  transform: scale(0.96);
}

.ck-ghost-btn {
  margin-top: 10px;
  height: 34px;
  padding: 0 20px;
  border: 0;
  border-radius: 9999px;
  background: var(--ck-bg-sunken);
  color: var(--ck-text);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}
.ck-ghost-btn:hover {
  background: var(--ck-border);
}

/* ---------- 骨架屏 ---------- */
.ck-home__loading {
  display: flex;
  flex-direction: column;
}
.ck-skel {
  display: flex;
  gap: 12px;
  padding: 16px;
  border-bottom: 1px solid var(--ck-divider);
}
.ck-skel__avatar {
  width: 42px;
  height: 42px;
  flex: none;
  border-radius: 9999px;
  background: var(--ck-bg-sunken);
  animation: ck-pulse 1.5s ease-in-out infinite;
}
.ck-skel__lines {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 9px;
  padding-top: 5px;
}
.ck-skel__line {
  height: 10px;
  border-radius: 5px;
  background: var(--ck-bg-sunken);
  animation: ck-pulse 1.5s ease-in-out infinite;
}
.ck-skel__line--short {
  width: 38%;
}
@keyframes ck-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.ck-home__sentinel {
  padding: 18px 0 8px;
  text-align: center;
  font-size: 12px;
  color: var(--ck-text-tertiary);
}

/* ---------- 悬浮发帖按钮 ---------- */
.ck-fab {
  position: absolute;
  right: 18px;
  bottom: 20px;
  width: 58px;
  height: 58px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 9999px;
  background: var(--ck-blue);
  color: #fff;
  cursor: pointer;
  box-shadow: var(--ck-shadow-float);
  z-index: 3;
  transition: transform 180ms var(--ck-ease-spring), background-color 160ms linear;
}
.ck-fab:hover {
  background: var(--ck-blue-hover);
}
.ck-fab:active {
  transform: scale(0.92);
}
</style>
