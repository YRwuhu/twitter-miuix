/**
 * 主题与本地偏好。不依赖后端，使用 localStorage 持久化。
 */
import { ref, watch } from 'vue'
import type { ThemeChoice } from './types'

const STORAGE_KEY = 'twitter-miuix.theme'
export const theme = ref<ThemeChoice>('light')

/** 主题名 → <html data-theme> 的值。CSS 变量按 dim/black 定义。 */
const DOM_THEME: Record<ThemeChoice, string> = {
  light: 'light',
  dark: 'dim',
  black: 'black',
}

function isTheme(value: string | null): value is ThemeChoice {
  return value === 'light' || value === 'dark' || value === 'black'
}

export function applyTheme(next: ThemeChoice): void {
  theme.value = next
  document.documentElement.dataset.theme = DOM_THEME[next]
  document.documentElement.classList.toggle('m-theme-dark', next !== 'light')
  document.documentElement.style.colorScheme = next === 'light' ? 'light' : 'dark'
}

export function initTheme(): void {
  let stored: string | null = null
  try {
    stored = localStorage.getItem(STORAGE_KEY)
  } catch {
    stored = null
  }

  const initial: ThemeChoice = isTheme(stored)
    ? stored
    : window.matchMedia?.('(prefers-color-scheme: dark)').matches
      ? 'dark'
      : 'light'

  applyTheme(initial)

  watch(theme, (value) => {
    try {
      localStorage.setItem(STORAGE_KEY, value)
    } catch {
      // 存储不可用时仅保留当前会话内的主题。
    }
  })
}