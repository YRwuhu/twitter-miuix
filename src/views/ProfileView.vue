<script setup lang="ts">
/** 个人页：真实账号资料；未登录时显示登录引导。 */
import { computed } from 'vue'
import { MiuixCard, MiuixTabRow, MiuixText } from 'miuix-vue'
import AppIcon from '../components/AppIcon.vue'
import AvatarBadge from '../components/AvatarBadge.vue'
import VerifiedBadge from '../components/VerifiedBadge.vue'
import { compactCount, type ThemeChoice, type XUser } from '../types'

const props = defineProps<{
  me: XUser | null
  loggedIn: boolean
  theme: ThemeChoice
}>()
const emit = defineEmits<{
  theme: [value: ThemeChoice]
  settings: []
  login: []
  logout: []
}>()

const themeTabs = ['浅色', '暗色', '纯黑']
const themeKeys: ThemeChoice[] = ['light', 'dark', 'black']
const themeIndex = computed(() => themeKeys.indexOf(props.theme))

const stats = computed(() => {
  const user = props.me
  if (!user) return []
  return [
    { label: '帖子', value: compactCount(user.tweets) || '0' },
    { label: '正在关注', value: compactCount(user.following) || '0' },
    { label: '关注者', value: compactCount(user.followers) || '0' },
  ]
})
</script>

<template>
  <div class="ck-me">
    <header class="ck-page-head">
      <h1 class="ck-page-title">我</h1>
      <button type="button" class="ck-page-head__icon" aria-label="设置" @click="emit('settings')">
        <AppIcon name="settings" :size="21" />
      </button>
    </header>

    <div class="ck-me__scroll">
      <div class="ck-me__content">
        <!-- 未登录 -->
        <div v-if="!loggedIn || !me" class="ck-empty">
          <div class="ck-empty__art">
            <AppIcon name="user" :size="32" :stroke-width="1.4" />
          </div>
          <h2 class="ck-empty__title">还没有登录</h2>
          <p class="ck-empty__desc">登录 X 账号后，这里会显示你的资料、关注数与帖子。</p>
          <button type="button" class="ck-primary-btn" @click="emit('login')">登录 X</button>
        </div>

        <template v-else>
          <MiuixCard class="ck-profile-card" press-feedback="none">
            <div class="ck-profile__banner">
              <img
                v-if="me.banner"
                :src="me.banner"
                alt=""
                class="ck-profile__banner-img"
                referrerpolicy="no-referrer"
              />
            </div>
            <div class="ck-profile__identity">
              <AvatarBadge :name="me.name" :avatar="me.avatar" :size="72" />
            </div>

            <div class="ck-profile__names">
              <div class="ck-profile__name-row">
                <strong>{{ me.name }}</strong>
                <VerifiedBadge :verified="me.verified" />
              </div>
              <span class="ck-profile__handle">@{{ me.handle }}</span>
              <p v-if="me.bio" class="ck-profile__bio">{{ me.bio }}</p>
            </div>

            <div v-if="stats.length" class="ck-profile__stats">
              <div v-for="stat in stats" :key="stat.label" class="ck-profile__stat">
                <strong>{{ stat.value }}</strong>
                <span>{{ stat.label }}</span>
              </div>
            </div>
          </MiuixCard>

          <MiuixCard class="ck-appearance-card" press-feedback="none">
            <MiuixText type="title4" weight="semibold" class="ck-card-title">外观</MiuixText>
            <div class="ck-appearance__tabrow">
              <MiuixTabRow
                :model-value="themeIndex"
                :tabs="themeTabs"
                @update:model-value="emit('theme', themeKeys[$event] ?? 'light')"
              />
            </div>
            <p class="ck-appearance__note">主题会保存到本地，下次启动自动恢复。</p>
          </MiuixCard>

          <MiuixCard class="ck-account-card" press-feedback="none">
            <MiuixText type="title4" weight="semibold" class="ck-card-title">账号</MiuixText>
            <button type="button" class="ck-account-card__row" @click="emit('logout')">
              <span>退出登录</span>
              <AppIcon name="arrowLeft" :size="15" />
            </button>
            <p class="ck-account-card__note">退出后内存中的 cookie 会立即清除。</p>
          </MiuixCard>
        </template>

        <div class="ck-me__footer">
          <span>twitter-miuix 0.1.1</span>
          <span>Rust + Tauri 2 + Vue 3 + miuix-vue</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ck-me {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--ck-bg);
}

.ck-page-head {
  flex: none;
  display: flex;
  align-items: center;
  padding: 10px 14px 6px 18px;
}
.ck-page-head__icon {
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
.ck-page-head__icon:hover {
  background: var(--ck-hover);
}

.ck-me__scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  scrollbar-width: none;
}
.ck-me__scroll::-webkit-scrollbar {
  width: 0;
}

.ck-me__content {
  display: flex;
  flex-direction: column;
  gap: var(--ck-page-gap);
  padding: 2px 10px 24px;
}

/* 未登录引导 */
.ck-me__gate-btn:hover {
  background: var(--ck-blue-hover);
}

.ck-profile-card {
  --m-card-color: var(--ck-card);
  box-shadow: var(--ck-shadow-card);
  overflow: hidden;
}

.ck-profile__banner {
  height: 92px;
  overflow: hidden;
  background: linear-gradient(120deg, #1d9bf0 0%, #7856ff 58%, #f91880 100%);
}
.ck-profile__banner-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.ck-profile__identity {
  display: flex;
  align-items: flex-end;
  padding: 0 14px;
  margin-top: -34px;
}
.ck-profile__identity :deep(.ck-avatar) {
  border: 4px solid var(--ck-card);
  box-sizing: content-box;
}

.ck-profile__names {
  padding: 8px 14px 0;
}
.ck-profile__name-row {
  display: flex;
  align-items: center;
  gap: 5px;
}
.ck-profile__name-row strong {
  font-size: 18px;
  font-weight: 800;
  color: var(--ck-text);
}
.ck-profile__handle {
  display: block;
  margin-top: 1px;
  font-size: 13px;
  color: var(--ck-text-secondary);
}
.ck-profile__bio {
  margin: 9px 0 0;
  font-size: 13px;
  line-height: 18px;
  color: var(--ck-text);
  white-space: pre-wrap;
  word-break: break-word;
}

.ck-profile__stats {
  display: flex;
  gap: 18px;
  padding: 12px 14px 14px;
}
.ck-profile__stat {
  display: inline-flex;
  gap: 4px;
  font-size: 12px;
  color: var(--ck-text-secondary);
}
.ck-profile__stat strong {
  color: var(--ck-text);
  font-weight: 700;
}

.ck-card-title {
  display: block;
  padding: 12px 14px 8px;
  color: var(--ck-text);
}

.ck-appearance-card,
.ck-account-card {
  --m-card-color: var(--ck-card);
  box-shadow: var(--ck-shadow-card);
}

.ck-appearance__tabrow {
  padding: 0 14px;
}
.ck-appearance__note {
  margin: 10px 0 0;
  padding: 0 14px 14px;
  font-size: 11px;
  color: var(--ck-text-secondary);
}

.ck-account-card__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 10px 14px;
  border: 0;
  border-top: 1px solid var(--ck-border);
  background: transparent;
  color: var(--ck-text);
  font-size: 13px;
  cursor: pointer;
}
.ck-account-card__row:hover {
  background: var(--ck-bg-sunken);
}
.ck-account-card__row :deep(svg) {
  transform: rotate(180deg);
  color: var(--ck-text-secondary);
}
.ck-account-card__note {
  margin: 0;
  padding: 0 14px 14px;
  font-size: 11px;
  color: var(--ck-text-tertiary);
}

.ck-me__footer {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 8px 6px 0;
  font-size: 10px;
  color: var(--ck-text-tertiary);
}
</style>
