<script setup lang="ts">
/** 底部导航：5 个标签页（主页/搜索/通知/消息/我）。 */
import { computed } from 'vue'
import AppIcon from './AppIcon.vue'
import AvatarBadge from './AvatarBadge.vue'
import type { TabKey, XUser } from '../types'

const props = defineProps<{ active: TabKey; unread: number; me?: XUser | null }>()
const emit = defineEmits<{ change: [key: TabKey] }>()

const items: { key: TabKey; label: string; icon: string }[] = [
  { key: 'home', label: '主页', icon: 'home' },
  { key: 'search', label: '搜索', icon: 'search' },
  { key: 'notifications', label: '通知', icon: 'bell' },
  { key: 'messages', label: '消息', icon: 'mail' },
  { key: 'me', label: '我', icon: 'user' },
]

const index = computed(() => items.findIndex((item) => item.key === props.active))

function select(key: TabKey): void {
  if (key !== props.active) emit('change', key)
}
</script>

<template>
  <nav class="ck-bottom">
    <button
      v-for="(item, i) in items"
      :key="item.key"
      class="ck-bottom__item"
      :class="{ 'is-selected': i === index }"
      type="button"
      :aria-label="item.label"
      :aria-selected="i === index"
      role="tab"
      @click="select(item.key)"
    >
      <span class="ck-bottom__icon-wrap" :class="{ 'is-selected': i === index }">
        <AvatarBadge
          v-if="item.key === 'me' && me"
          :name="me.name"
          :avatar="me.avatar"
          :size="24"
        />
        <AppIcon v-else :name="item.icon" :size="22" :filled="i === index" />
      </span>
      <span v-if="item.key === 'messages' && unread > 0" class="ck-bottom__badge">{{ unread }}</span>
    </button>
  </nav>
</template>

<style scoped>
.ck-bottom {
  flex: none;
  display: flex;
  align-items: center;
  justify-content: space-around;
  min-height: 56px;
  padding: 0 4px;
  background: var(--ck-bg-elevated);
  border-top: 1px solid var(--ck-border);
  position: relative;
  z-index: 4;
}

.ck-bottom__item {
  position: relative;
  flex: 1;
  height: 56px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  background: transparent;
  color: var(--ck-text-secondary);
  cursor: pointer;
  transition: color 180ms var(--ck-ease-spring);
}
.ck-bottom__item:hover {
  color: var(--ck-text);
}
.ck-bottom__item.is-selected {
  color: var(--ck-blue);
}

/* HyperOS 风格：选中项图标带柔和的胶囊底 */
.ck-bottom__icon-wrap {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 32px;
  border-radius: 9999px;
  transition: background-color 220ms var(--ck-ease-spring), transform 220ms var(--ck-ease-spring);
}
.ck-bottom__icon-wrap.is-selected {
  background: var(--ck-active);
  transform: scale(1.02);
}
.ck-bottom__item:active .ck-bottom__icon-wrap {
  transform: scale(0.92);
}

.ck-bottom__badge {
  position: absolute;
  top: 11px;
  left: calc(50% + 7px);
  min-width: 15px;
  height: 15px;
  padding: 0 4px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 9999px;
  background: var(--ck-blue);
  color: #fff;
  font-size: 9px;
  font-weight: 700;
  line-height: 1;
  border: 2px solid var(--ck-bg-elevated);
  box-sizing: content-box;
}
</style>