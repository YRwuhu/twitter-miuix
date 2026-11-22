/**
 * 时间线 store：双时间线、游标分页、加载/错误态。
 *
 * 数据来自 Rust 侧的 `x_home_timeline` 命令。未登录时会返回
 * `kind: 'unauthorized'`，由 UI 提示登录。
 */
import { defineStore } from 'pinia'
import { computed, reactive, ref } from 'vue'
import { homeTimeline, toXApiError } from '../api/x'
import type { TimelineKind, XPost } from '../types'

export interface TimelineState {
  posts: XPost[]
  bottomCursor: string | null
  loading: boolean
  loadingMore: boolean
  error: string | null
  /** 是否已经拉到末尾 */
  done: boolean
  loadedOnce: boolean
}

function emptyState(): TimelineState {
  return {
    posts: [],
    bottomCursor: null,
    loading: false,
    loadingMore: false,
    error: null,
    done: false,
    loadedOnce: false,
  }
}

export const useTimelineStore = defineStore('timeline', () => {
  const forYou = reactive<TimelineState>(emptyState())
  const following = reactive<TimelineState>(emptyState())
  const active = ref<TimelineKind>('forYou')
  const requestVersion: Record<TimelineKind, number> = { forYou: 0, following: 0 }

  const state = computed(() => (active.value === 'forYou' ? forYou : following))

  function target(kind: TimelineKind): TimelineState {
    return kind === 'forYou' ? forYou : following
  }

  /** 首屏或刷新。`force` 为 true 时即使已有数据也重拉。 */
  async function load(kind: TimelineKind, force = false): Promise<void> {
    const s = target(kind)
    if (s.loading) return
    if (s.loadedOnce && !force) return

    const version = ++requestVersion[kind]
    s.loading = true
    s.error = null
    try {
      const page = await homeTimeline(kind, null)
      if (version !== requestVersion[kind]) return
      s.posts = dedupe(page.posts)
      s.bottomCursor = page.bottomCursor ?? null
      s.done = !page.bottomCursor
      s.loadedOnce = true
    } catch (error) {
      if (version !== requestVersion[kind]) return
      const e = toXApiError(error)
      s.error = describeError(e.kind, e.message)
    } finally {
      if (version === requestVersion[kind]) s.loading = false
    }
  }

  /** 滚动到底部时加载下一页。 */
  async function loadMore(kind: TimelineKind): Promise<void> {
    const s = target(kind)
    if (s.loading || s.loadingMore || s.done) return
    if (!s.bottomCursor) {
      s.done = true
      return
    }

    const version = requestVersion[kind]
    s.loadingMore = true
    s.error = null
    try {
      const page = await homeTimeline(kind, s.bottomCursor)
      if (version !== requestVersion[kind]) return
      s.posts = dedupe([...s.posts, ...page.posts])
      s.bottomCursor = page.bottomCursor ?? null
      s.done = !page.bottomCursor || page.posts.length === 0
    } catch (error) {
      if (version !== requestVersion[kind]) return
      const e = toXApiError(error)
      s.error = describeError(e.kind, e.message)
    } finally {
      if (version === requestVersion[kind]) s.loadingMore = false
    }
  }

  /** 下拉刷新：清空后重拉。 */
  async function refresh(kind: TimelineKind): Promise<void> {
    const s = target(kind)
    requestVersion[kind]++
    s.loading = false
    s.loadingMore = false
    s.posts = []
    s.bottomCursor = null
    s.done = false
    s.loadedOnce = false
    await load(kind, true)
  }

  /** 切换时间线标签。 */
  async function switchTo(kind: TimelineKind): Promise<void> {
    active.value = kind
    const s = target(kind)
    if (!s.loadedOnce && !s.loading) {
      await load(kind)
    }
  }

  /** 本地更新已加载帖子的状态（点赞/转推/书签后同步）。 */
  function patchPost(id: string, patch: Partial<XPost>): void {
    for (const s of [forYou, following]) {
      const index = s.posts.findIndex((p) => p.id === id)
      if (index >= 0) {
        s.posts[index] = { ...s.posts[index], ...patch }
      }
    }
  }

  /** 把新发布的帖子插到「正在关注」顶部。 */
  function prependPost(post: XPost): void {
    following.posts = dedupe([post, ...following.posts])
  }

  function reset(): void {
    requestVersion.forYou++
    requestVersion.following++
    Object.assign(forYou, emptyState())
    Object.assign(following, emptyState())
  }

  /** 去重：时间线里同一帖子可能因分页重叠出现两次。 */
  function dedupe(list: XPost[]): XPost[] {
    const seen = new Set<string>()
    const out: XPost[] = []
    for (const post of list) {
      if (seen.has(post.id)) continue
      seen.add(post.id)
      out.push(post)
    }
    return out
  }

  /** 把错误码翻译成用户能理解的话。 */
  function describeError(kind: string, message: string): string {
    switch (kind) {
      case 'unauthorized':
        return '需要登录 X 账号才能读取时间线'
      case 'rate_limited':
        return '请求过于频繁，请稍后再试'
      case 'not_found':
        return '接口不可用：可能需要更新 queryId，或登录态已失效'
      case 'network':
        return '网络连接失败，请检查网络后重试'
      case 'unsupported':
        return message
      default:
        return message || '加载失败'
    }
  }

  return {
    forYou,
    following,
    active,
    state,
    load,
    loadMore,
    refresh,
    switchTo,
    patchPost,
    prependPost,
    reset,
  }
})
