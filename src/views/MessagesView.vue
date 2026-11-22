<script setup lang="ts">
/** 私信：会话列表 + 单聊消息。 */
import { nextTick, onUnmounted, ref, watch } from 'vue'
import { MiuixCard, MiuixText } from 'miuix-vue'
import AppIcon from '../components/AppIcon.vue'
import AvatarBadge from '../components/AvatarBadge.vue'
import { dmConversations, dmMessages, dmSend, toXApiError } from '../api/x'
import { relativeTime, type DmConversation, type DmMessage, type XUser } from '../types'

const props = defineProps<{ me: XUser | null; loggedIn: boolean }>()
const emit = defineEmits<{ login: []; openUser: [user: XUser] }>()

const conversations = ref<DmConversation[]>([])
const loading = ref(false)
const error = ref('')
const loaded = ref(false)

const active = ref<DmConversation | null>(null)
const messages = ref<DmMessage[]>([])
const msgLoading = ref(false)
const msgError = ref('')
const draft = ref('')
const sending = ref(false)
const scroller = ref<HTMLElement | null>(null)
let conversationRequest = 0
let messageRequest = 0

function title(c: DmConversation): string {
  if (!c.participants.length) return '私信会话'
  return c.participants.map((p) => p.name).join('、')
}

function avatar(c: DmConversation): XUser | null {
  return c.participants[0] ?? null
}

async function load(): Promise<void> {
  if (!props.loggedIn || !props.me?.id) return
  const request = ++conversationRequest
  loading.value = true
  error.value = ''
  try {
    conversations.value = await dmConversations(props.me.id)
    if (request !== conversationRequest) return
    loaded.value = true
  } catch (e) {
    if (request !== conversationRequest) return
    const err = toXApiError(e)
    error.value = err.kind === 'unauthorized' ? '需要登录 X 账号' : err.message
  } finally {
    if (request === conversationRequest) loading.value = false
  }
}

async function open(c: DmConversation): Promise<void> {
  const request = ++messageRequest
  sending.value = false
  active.value = c
  messages.value = []
  msgError.value = ''
  if (!props.me?.id) return
  msgLoading.value = true
  try {
    messages.value = await dmMessages(c.id, props.me.id)
    if (request !== messageRequest) return
    await scrollBottom()
  } catch (e) {
    if (request === messageRequest) msgError.value = toXApiError(e).message
  } finally {
    if (request === messageRequest) msgLoading.value = false
  }
}

function back(): void {
  messageRequest++
  sending.value = false
  active.value = null
  messages.value = []
  draft.value = ''
}

async function send(): Promise<void> {
  const text = draft.value.trim()
  if (!text || !active.value || sending.value || !props.me?.id) return
  const conversationId = active.value.id
  const userId = props.me.id
  const request = ++messageRequest
  sending.value = true
  try {
    await dmSend(conversationId, text)
    if (request !== messageRequest || active.value?.id !== conversationId) return
    draft.value = ''
    messages.value = await dmMessages(conversationId, userId)
    if (request !== messageRequest || active.value?.id !== conversationId) return
    await scrollBottom()
  } catch (e) {
    if (request === messageRequest) msgError.value = toXApiError(e).message
  } finally {
    if (request === messageRequest) sending.value = false
  }
}

async function scrollBottom(): Promise<void> {
  await nextTick()
  if (scroller.value) scroller.value.scrollTop = scroller.value.scrollHeight
}

watch(() => [props.loggedIn, props.me?.id] as const, ([loggedIn, id]) => {
  conversationRequest++
  messageRequest++
  sending.value = false
  active.value = null
  messages.value = []
  conversations.value = []
  loaded.value = false
  error.value = ''
  if (loggedIn && id) void load()
  else loading.value = false
}, { immediate: true })

onUnmounted(() => {
  conversationRequest++
  messageRequest++
})
</script>

<template>
  <div class="ck-dm">
    <!-- 会话列表 -->
    <template v-if="!active">
      <header class="ck-dm__head">
        <h1 class="ck-page-title">消息</h1>
      </header>

      <div class="ck-dm__scroll">
        <div class="ck-dm__content">
          <div v-if="!loggedIn" class="ck-empty">
            <div class="ck-empty__art">
              <AppIcon name="mail" :size="32" :stroke-width="1.4" />
            </div>
            <h2 class="ck-empty__title">还没有登录</h2>
            <p class="ck-empty__desc">登录 X 账号后，这里会显示你的私信。</p>
            <button type="button" class="ck-primary-btn" @click="emit('login')">登录 X</button>
          </div>

          <div v-else-if="loading" class="ck-dm__status">加载中…</div>

          <div v-else-if="error" class="ck-empty">
            <p class="ck-empty__desc">{{ error }}</p>
            <button type="button" class="ck-ghost-btn" @click="load">重试</button>
          </div>

          <div v-else-if="loaded && !conversations.length" class="ck-empty">
            <div class="ck-empty__art">
              <AppIcon name="mail" :size="32" :stroke-width="1.4" />
            </div>
            <h2 class="ck-empty__title">还没有会话</h2>
            <p class="ck-empty__desc">收到私信后会显示在这里。</p>
          </div>

          <MiuixCard
            v-for="c in conversations"
            :key="c.id"
            class="ck-dm__conv"
            press-feedback="none"
            show-indication
          >
            <button type="button" class="ck-dm__conv-btn" @click="open(c)">
              <AvatarBadge
                :name="avatar(c)?.name || '?'"
                :avatar="avatar(c)?.avatar"
                :size="44"
              />
              <div class="ck-dm__conv-body">
                <div class="ck-dm__conv-head">
                  <strong>{{ title(c) }}</strong>
                  <time v-if="c.lastTime">{{ relativeTime(c.lastTime) }}</time>
                </div>
                <p>{{ c.lastMessage || '暂无消息' }}</p>
              </div>
            </button>
          </MiuixCard>
        </div>
      </div>
    </template>

    <!-- 聊天 -->
    <template v-else>
      <header class="ck-dm__head ck-dm__head--chat">
        <button type="button" class="ck-dm__back" aria-label="返回" @click="back">
          <AppIcon name="arrowLeft" :size="20" />
        </button>
        <MiuixText type="title3" weight="medium" class="ck-dm__title ck-dm__title--chat">
          {{ title(active) }}
        </MiuixText>
        <button
          v-if="avatar(active)"
          type="button"
          class="ck-dm__back"
          aria-label="查看资料"
          @click="emit('openUser', avatar(active)!)"
        >
          <AppIcon name="user" :size="19" />
        </button>
        <span v-else class="ck-dm__back-space" />
      </header>

      <div ref="scroller" class="ck-dm__chat">
        <div v-if="msgLoading" class="ck-dm__status">加载消息…</div>
        <MiuixCard v-else-if="msgError" class="ck-dm__state" press-feedback="none">
          <p class="ck-dm__error">{{ msgError }}</p>
          <button type="button" class="ck-dm__btn" @click="open(active)">重试</button>
        </MiuixCard>
        <p v-else-if="!messages.length" class="ck-dm__empty">还没有消息</p>

        <div v-else class="ck-dm__bubbles">
          <div
            v-for="m in messages"
            :key="m.id"
            class="ck-dm__bubble"
            :class="{ 'is-me': m.fromMe }"
          >
            <p>{{ m.text }}</p>
            <time v-if="m.time">{{ relativeTime(m.time) }}</time>
          </div>
        </div>
      </div>

      <footer class="ck-dm__composer">
        <input
          v-model="draft"
          class="ck-dm__input"
          type="text"
          placeholder="发送私信…"
          @keydown.enter="send"
        />
        <button
          type="button"
          class="ck-dm__send"
          :disabled="sending || !draft.trim()"
          aria-label="发送"
          @click="send"
        >
          <AppIcon name="share" :size="17" />
        </button>
      </footer>
    </template>
  </div>
</template>

<style scoped>
.ck-dm {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--ck-bg);
}

.ck-dm__head {
  flex: none;
  padding: 10px 18px 8px;
}
.ck-dm__head--chat {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 56px;
  padding: 0 8px;
}
.ck-dm__title--chat {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  max-width: 56%;
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  pointer-events: none;
}
.ck-dm__back {
  width: 34px;
  height: 34px;
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 9999px;
  background: transparent;
  color: var(--ck-text);
  cursor: pointer;
}
.ck-dm__back-space {
  width: 34px;
  flex: none;
}

.ck-dm__scroll,
.ck-dm__chat {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  scrollbar-width: none;
}
.ck-dm__scroll::-webkit-scrollbar,
.ck-dm__chat::-webkit-scrollbar {
  width: 0;
}

.ck-dm__content {
  display: flex;
  flex-direction: column;
  gap: var(--ck-page-gap);
  padding: 0 10px 24px;
}


.ck-dm__status,
.ck-dm__empty {
  padding: 30px 0;
  text-align: center;
  font-size: 12px;
  color: var(--ck-text-secondary);
}

.ck-dm__conv {
  --m-card-color: var(--ck-card);
  box-shadow: var(--ck-shadow-card);
  padding: 4px 0;
}
.ck-dm__conv-btn {
  display: flex;
  align-items: center;
  gap: 11px;
  width: 100%;
  padding: 11px 14px;
  border: 0;
  background: transparent;
  text-align: left;
  cursor: pointer;
}
.ck-dm__conv-body {
  flex: 1;
  min-width: 0;
}
.ck-dm__conv-head {
  display: flex;
  align-items: baseline;
  gap: 8px;
}
.ck-dm__conv-head strong {
  flex: 1;
  min-width: 0;
  font-size: 14px;
  color: var(--ck-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.ck-dm__conv-head time {
  flex: none;
  font-size: 11px;
  color: var(--ck-text-tertiary);
}
.ck-dm__conv-body p {
  margin: 3px 0 0;
  font-size: 12px;
  line-height: 17px;
  color: var(--ck-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 聊天气泡 */
.ck-dm__bubbles {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 14px 20px;
}
.ck-dm__bubble {
  align-self: flex-start;
  max-width: 78%;
  padding: 9px 12px;
  border-radius: 16px 16px 16px 4px;
  background: var(--ck-card);
}
.ck-dm__bubble.is-me {
  align-self: flex-end;
  border-radius: 16px 16px 4px 16px;
  background: var(--ck-blue);
  color: #fff;
}
.ck-dm__bubble p {
  margin: 0;
  font-size: 13px;
  line-height: 18px;
  white-space: pre-wrap;
  word-break: break-word;
}
.ck-dm__bubble time {
  display: block;
  margin-top: 3px;
  font-size: 9px;
  opacity: 0.6;
}

.ck-dm__composer {
  flex: none;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px 12px;
  background: var(--ck-bg);
}
.ck-dm__input {
  flex: 1;
  min-width: 0;
  height: 38px;
  padding: 0 14px;
  border: 1px solid var(--ck-border);
  border-radius: 9999px;
  background: var(--ck-card);
  color: var(--ck-text);
  font-size: 13px;
  outline: none;
}
.ck-dm__input:focus {
  border-color: var(--ck-blue);
}
.ck-dm__send {
  width: 38px;
  height: 38px;
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 9999px;
  background: var(--ck-blue);
  color: #fff;
  cursor: pointer;
}
.ck-dm__send:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
