<script setup lang="ts">
/** 通知页：真实通知流。 */
import { onMounted, ref } from 'vue'
import { MiuixCard, MiuixTabRow, MiuixText } from 'miuix-vue'
import AppIcon from '../components/AppIcon.vue'
import AvatarBadge from '../components/AvatarBadge.vue'
import { notifications as fetchNotifications, toXApiError } from '../api/x'
import { relativeTime, type XNotification, type XUser } from '../types'

const props = defineProps<{
  me: XUser | null
  loggedIn: boolean
}>()
const emit = defineEmits<{ open: [id: string]; login: [] }>()

const tabs = ['全部', '提及']
const tabIndex = ref(0)
const items = ref<XNotification[]>([])
const loading = ref(false)
const error = ref('')
const loaded = ref(false)

const ICONS: Record<string, { name: string; color: string }> = {
  like: { name: 'heart', color: 'var(--ck-pink)' },
  repost: { name: 'repost', color: 'var(--ck-green)' },
  follow: { name: 'user', color: 'var(--ck-blue)' },
  mention: { name: 'comment', color: 'var(--ck-blue)' },
  reply: { name: 'comment', color: 'var(--ck-blue)' },
  quote: { name: 'repost', color: 'var(--ck-green)' },
}

function iconStyle(kind: string): Record<string, string> {
  const icon = ICONS[kind] ?? { name: 'bell', color: 'rgba(139,152,165,1)' }
  return { color: icon.color, background: `color-mix(in srgb, ${icon.color} 12%, transparent)` }
}

function iconName(kind: string): string {
  return (ICONS[kind] ?? { name: 'bell' }).name
}

async function load(): Promise<void> {
  if (!props.loggedIn || !props.me?.id) return
  loading.value = true
  error.value = ''
  try {
    items.value = await fetchNotifications(
      props.me.id,
      tabIndex.value === 0 ? 'All' : 'Mentions',
    )
    loaded.value = true
  } catch (e) {
    const err = toXApiError(e)
    error.value = err.kind === 'unauthorized' ? '需要登录 X 账号' : err.message
  } finally {
    loading.value = false
  }
}

function onTabChange(value: number): void {
  tabIndex.value = value
  void load()
}

onMounted(() => {
  void load()
})
</script>

<template>
  <div class="ck-notif">
    <header class="ck-notif__head">
      <h1 class="ck-page-title">通知</h1>
    </header>

    <div v-if="loggedIn" class="ck-notif__tabs">
      <MiuixTabRow :model-value="tabIndex" :tabs="tabs" @update:model-value="onTabChange" />
    </div>

    <div class="ck-notif__scroll">
      <div class="ck-notif__content">
        <div v-if="!loggedIn" class="ck-empty">
          <div class="ck-empty__art"><AppIcon name="bell" :size="32" :stroke-width="1.4" /></div>
          <h2 class="ck-empty__title">还没有登录</h2>
          <p class="ck-empty__desc">登录 X 账号后，这里会显示你的通知。</p>
          <button type="button" class="ck-primary-btn" @click="emit('login')">登录 X</button>
        </div>

        <div v-else-if="loading" class="ck-notif__loading">
          <div v-for="i in 4" :key="i" class="ck-notif__skeleton" />
        </div>

        <div v-else-if="error" class="ck-empty">
          <p class="ck-empty__desc">{{ error }}</p>
          <button type="button" class="ck-ghost-btn" @click="load">重试</button>
        </div>

        <div v-else-if="loaded && !items.length" class="ck-empty">
          <div class="ck-empty__art"><AppIcon name="bell" :size="32" :stroke-width="1.4" /></div>
          <h2 class="ck-empty__title">暂无通知</h2>
          <p class="ck-empty__desc">有新互动时会显示在这里。</p>
        </div>

        <MiuixCard
          v-for="item in items"
          :key="item.id"
          class="ck-notif__card"
          press-feedback="none"
          show-indication
        >
          <div class="ck-notif__row" @click="item.post && emit('open', item.post.id)">
            <span class="ck-notif__icon" :style="iconStyle(item.kind)">
              <AppIcon :name="iconName(item.kind)" :size="17" />
            </span>

            <div class="ck-notif__body">
              <p class="ck-notif__message">{{ item.message || '有一条新通知' }}</p>
              <p v-if="item.post" class="ck-notif__excerpt">{{ item.post.text }}</p>
            </div>

            <time v-if="item.createdAt" class="ck-notif__time">
              {{ relativeTime(item.createdAt) }}
            </time>
          </div>

          <button
            v-if="item.post"
            type="button"
            class="ck-notif__post"
            @click="emit('open', item.post.id)"
          >
            <AvatarBadge
              :name="item.post.author.name"
              :avatar="item.post.author.avatar"
              :size="20"
            />
            <span>@{{ item.post.author.handle }}</span>
          </button>
        </MiuixCard>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ck-notif {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--ck-bg);
}

.ck-notif__head {
  flex: none;
  padding: 10px 18px 6px;
}

.ck-notif__tabs {
  flex: none;
  padding: 2px 16px 10px;
}

.ck-notif__scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  scrollbar-width: none;
}
.ck-notif__scroll::-webkit-scrollbar {
  width: 0;
}

.ck-notif__content {
  display: flex;
  flex-direction: column;
  gap: var(--ck-page-gap);
  padding: 0 10px 24px;
}


.ck-notif__loading {
  display: flex;
  flex-direction: column;
  gap: var(--ck-page-gap);
}
.ck-notif__skeleton {
  height: 72px;
  border-radius: var(--ck-radius-card);
  background: var(--ck-card);
  animation: ck-notif-pulse 1.4s ease-in-out infinite;
}
@keyframes ck-notif-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.45; }
}

.ck-notif__card {
  --m-card-color: var(--ck-card);
  box-shadow: var(--ck-shadow-card);
  padding: 12px 14px;
}

.ck-notif__row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  cursor: pointer;
}

.ck-notif__icon {
  width: 32px;
  height: 32px;
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 9999px;
}

.ck-notif__body {
  flex: 1;
  min-width: 0;
}
.ck-notif__message {
  margin: 0;
  font-size: 13px;
  line-height: 19px;
  color: var(--ck-text);
  font-weight: 600;
}
.ck-notif__excerpt {
  margin: 3px 0 0;
  font-size: 12px;
  line-height: 17px;
  color: var(--ck-text-secondary);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.ck-notif__time {
  flex: none;
  font-size: 11px;
  color: var(--ck-text-tertiary);
}

.ck-notif__post {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 8px 0 0 42px;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--ck-text-secondary);
  font-size: 11px;
  cursor: pointer;
}
.ck-notif__post:hover {
  color: var(--ck-blue);
}
</style>