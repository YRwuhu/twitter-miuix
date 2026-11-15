<script setup lang="ts">
/** 登录面板：粘贴 cookie，或用游客身份浏览公开数据。 */
import { computed, ref } from 'vue'
import { MiuixBottomSheet, MiuixButton, MiuixText } from 'miuix-vue'
import { activateGuest, login, toXApiError } from '../api/x'
import type { AuthStatus } from '../api/x'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  success: [status: AuthStatus]
}>()

const cookie = ref('')
const busy = ref(false)
const error = ref('')
const hint = ref('')

const open = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value),
})

const canSubmit = computed(() => cookie.value.trim().length > 0 && !busy.value)

async function doLogin(): Promise<void> {
  if (!canSubmit.value) return
  busy.value = true
  error.value = ''
  hint.value = ''
  try {
    const status = await login(cookie.value.trim())
    cookie.value = ''
    emit('success', status)
    open.value = false
  } catch (e) {
    const err = toXApiError(e)
    error.value = err.message
  } finally {
    busy.value = false
  }
}

async function doGuest(): Promise<void> {
  busy.value = true
  error.value = ''
  try {
    const status = await activateGuest()
    emit('success', status)
    hint.value = ''
    open.value = false
  } catch (e) {
    error.value = toXApiError(e).message
  } finally {
    busy.value = false
  }
}

function close(): void {
  open.value = false
  error.value = ''
}
</script>

<template>
  <MiuixBottomSheet v-model="open" title="登录 X" class="ck-login">
    <template #end-action>
      <MiuixButton type="default" class="ck-login__small" @click="close">关闭</MiuixButton>
    </template>

    <div class="ck-login__body">
      <MiuixText type="body2" class="ck-login__lead">
        时间线需要登录态。粘贴已登录 x.com 浏览器的 cookie 即可，凭据只存在内存，关闭应用就清除。
      </MiuixText>

      <label class="ck-login__label">cookie</label>
      <textarea
        v-model="cookie"
        class="ck-login__input"
        rows="4"
        spellcheck="false"
        placeholder="auth_token=...; ct0=..."
        @keydown.ctrl.enter="doLogin"
      />

      <p class="ck-login__tip">
        获取方式：浏览器登录 x.com → F12 → Application → Cookies → https://x.com，
        复制 <code>auth_token</code> 与 <code>ct0</code> 两行的值，按上面的格式拼成一行。
      </p>

      <p v-if="error" class="ck-login__error">{{ error }}</p>
      <p v-if="hint" class="ck-login__hint">{{ hint }}</p>

      <div class="ck-login__actions">
        <button
          type="button"
          class="ck-login__btn ck-login__btn--primary"
          :disabled="!canSubmit"
          @click="doLogin"
        >
          {{ busy ? '验证中…' : '登录' }}
        </button>
        <button type="button" class="ck-login__btn" :disabled="busy" @click="doGuest">
          游客身份
        </button>
      </div>

      <p class="ck-login__note">
        游客身份只能读取公开数据（用户资料、趋势），看不到你自己的时间线。
      </p>
    </div>
  </MiuixBottomSheet>
</template>

<style scoped>
.ck-login :deep(.m-bottom-sheet__inner) {
  background: var(--ck-card);
}

.ck-login__small {
  min-height: 30px;
  height: 30px;
  padding: 0 12px;
  font-size: 12px;
}

.ck-login__body {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-bottom: 12px;
}

.ck-login__lead {
  color: var(--ck-text-secondary);
  line-height: 19px;
}

.ck-login__label {
  margin-top: 6px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--ck-text-secondary);
}

.ck-login__input {
  width: 100%;
  resize: vertical;
  min-height: 82px;
  padding: 10px 12px;
  border: 1px solid var(--ck-border);
  border-radius: 12px;
  background: var(--ck-bg-sunken);
  color: var(--ck-text);
  font-size: 12px;
  line-height: 18px;
  font-family: ui-monospace, 'Cascadia Mono', monospace;
  outline: none;
}
.ck-login__input:focus {
  border-color: var(--ck-blue);
}

.ck-login__tip,
.ck-login__note {
  margin: 0;
  font-size: 10px;
  line-height: 16px;
  color: var(--ck-text-secondary);
}
.ck-login__tip code {
  font-family: ui-monospace, monospace;
  background: var(--ck-bg-sunken);
  padding: 1px 4px;
  border-radius: 4px;
}

.ck-login__error {
  margin: 0;
  padding: 8px 10px;
  border-radius: 10px;
  background: rgba(244, 33, 46, 0.1);
  color: var(--ck-red);
  font-size: 11px;
  line-height: 16px;
}

.ck-login__hint {
  margin: 0;
  font-size: 11px;
  color: var(--ck-text-secondary);
}

.ck-login__actions {
  display: flex;
  gap: 8px;
  margin-top: 6px;
}

.ck-login__btn {
  flex: 1;
  height: 38px;
  border: 0;
  border-radius: 9999px;
  background: var(--ck-bg-sunken);
  color: var(--ck-text);
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  transition: background-color 120ms linear;
}
.ck-login__btn:hover:not(:disabled) {
  background: var(--ck-border);
}
.ck-login__btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.ck-login__btn--primary {
  background: var(--ck-blue);
  color: #fff;
}
.ck-login__btn--primary:hover:not(:disabled) {
  background: var(--ck-blue-hover);
}
</style>