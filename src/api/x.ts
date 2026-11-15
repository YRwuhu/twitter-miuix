/**
 * 真实 X 数据层的前端封装。
 *
 * 网络请求全部在 Rust 侧完成，前端只通过 invoke 调用命令。
 * 凭据仅存在 Rust 内存，退出即清空。
 */
import { invoke } from '@tauri-apps/api/core'
import type { DmConversation, DmMessage, TimelineKind, TimelinePage, UserTimelineKind, XNotification, XUser } from '../types'

export interface AuthStatus {
  logged_in: boolean
  guest: boolean
  user?: XUser | null
}

/** Rust 侧返回的结构化错误。 */
export interface XApiError {
  message: string
  status: number
  /** unauthorized | not_found | rate_limited | server | network | parse
   *  | graphql_error | bad_variables | unknown_operation | bad_credentials | unsupported */
  kind: string
}

export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

function parseError(raw: unknown): XApiError {
  if (typeof raw === 'object' && raw !== null) {
    const obj = raw as Record<string, unknown>
    if (typeof obj.kind === 'string') {
      return {
        message: typeof obj.message === 'string' ? obj.message : String(raw),
        status: typeof obj.status === 'number' ? obj.status : 0,
        kind: obj.kind,
      }
    }
  }
  if (typeof raw === 'string') {
    try {
      const parsed = JSON.parse(raw)
      if (parsed && typeof parsed === 'object' && 'kind' in parsed) {
        return parsed as XApiError
      }
      return { message: raw, status: 0, kind: 'unknown' }
    } catch {
      return { message: raw, status: 0, kind: 'unknown' }
    }
  }
  return { message: String(raw), status: 0, kind: 'unknown' }
}

export function toXApiError(error: unknown): XApiError {
  return parseError(error)
}

/** 浏览器预览环境下没有 Tauri 命令。 */
function offlineError(action: string): XApiError {
  return { message: `${action}需要在桌面应用内运行`, status: 0, kind: 'unsupported' }
}

export async function authStatus(): Promise<AuthStatus> {
  if (!isTauri()) return { logged_in: false, guest: false, user: null }
  return invoke<AuthStatus>('x_auth_status')
}

/** 用 cookie 登录。Rust 侧会先用 UserByScreenName 验证，无效 cookie 直接报错。 */
export async function login(cookie: string): Promise<AuthStatus> {
  if (!isTauri()) throw offlineError('登录')
  return invoke<AuthStatus>('x_login', { cookie })
}

export async function logout(): Promise<AuthStatus> {
  if (!isTauri()) return { logged_in: false, guest: false, user: null }
  return invoke<AuthStatus>('x_logout')
}

export async function activateGuest(): Promise<AuthStatus> {
  if (!isTauri()) throw offlineError('游客身份')
  return invoke<AuthStatus>('x_activate_guest')
}

export async function setQueryId(operation: string, queryId: string): Promise<void> {
  if (!isTauri()) throw offlineError('设置 queryId')
  await invoke('x_set_query_id', { operation, queryId })
}

export async function queryIds(): Promise<Record<string, string>> {
  if (!isTauri()) return {}
  return invoke<Record<string, string>>('x_query_ids')
}

/** 拉取首页时间线。`cursor` 为空表示首屏。 */
export async function homeTimeline(
  kind: TimelineKind,
  cursor?: string | null,
  count = 20,
): Promise<TimelinePage> {
  if (!isTauri()) throw offlineError('时间线')
  return invoke<TimelinePage>('x_home_timeline', { kind, cursor: cursor ?? null, count })
}

export async function currentUser(): Promise<XUser> {
  if (!isTauri()) throw offlineError('当前用户')
  return invoke<XUser>('x_current_user')
}

/** 点赞 / 取消点赞（真实 mutation）。 */
export async function favorite(tweetId: string, on: boolean): Promise<void> {
  if (!isTauri()) throw offlineError('点赞')
  await invoke('x_favorite', { tweetId, on })
}

/** 转推 / 取消转推。 */
export async function retweet(tweetId: string, on: boolean): Promise<void> {
  if (!isTauri()) throw offlineError('转推')
  await invoke('x_retweet', { tweetId, on })
}

/** 加入 / 移出书签。 */
export async function bookmark(tweetId: string, on: boolean): Promise<void> {
  if (!isTauri()) throw offlineError('书签')
  await invoke('x_bookmark', { tweetId, on })
}

/** 发布帖子或回复。 */
export async function createTweet(text: string, replyTo?: string | null): Promise<unknown> {
  if (!isTauri()) throw offlineError('发帖')
  return invoke('x_create_tweet', { text, replyTo: replyTo ?? null })
}

/** 用户主页时间线。 */
export async function userTimeline(
  userId: string,
  kind: UserTimelineKind,
  cursor?: string | null,
  count = 20,
): Promise<TimelinePage> {
  if (!isTauri()) throw offlineError('用户主页')
  return invoke<TimelinePage>('x_user_timeline', { userId, kind, cursor: cursor ?? null, count })
}

/** 书签时间线。 */
export async function bookmarks(cursor?: string | null, count = 20): Promise<TimelinePage> {
  if (!isTauri()) throw offlineError('书签')
  return invoke<TimelinePage>('x_bookmarks', { cursor: cursor ?? null, count })
}

/** 搜索帖子。`product` 为 Latest 或 Top。 */
export async function searchTimeline(
  query: string,
  product: 'Latest' | 'Top' = 'Latest',
  cursor?: string | null,
  count = 20,
): Promise<TimelinePage> {
  if (!isTauri()) throw offlineError('搜索')
  return invoke<TimelinePage>('x_search_timeline', { query, product, cursor: cursor ?? null, count })
}

/** 通知时间线。 */
export async function notifications(
  userId: string,
  kind: 'All' | 'Mentions' = 'All',
  count = 20,
): Promise<XNotification[]> {
  if (!isTauri()) throw offlineError('通知')
  return invoke<XNotification[]>('x_notifications', { userId, kind, count })
}

/** 帖子详情：返回 `[主帖, ...回复]`。 */
export async function tweetDetail(tweetId: string): Promise<TimelinePage> {
  if (!isTauri()) throw offlineError('帖子详情')
  return invoke<TimelinePage>('x_tweet_detail', { tweetId })
}

/** 启动时恢复上次登录态。 */
export async function restoreSession(): Promise<AuthStatus> {
  if (!isTauri()) return { logged_in: false, guest: false, user: null }
  return invoke<AuthStatus>('x_restore_session')
}

/** 私信会话列表。 */
export async function dmConversations(myId: string): Promise<DmConversation[]> {
  if (!isTauri()) throw offlineError('私信')
  return invoke<DmConversation[]>('x_dm_conversations', { myId })
}

/** 会话消息。 */
export async function dmMessages(conversationId: string, myId: string): Promise<DmMessage[]> {
  if (!isTauri()) throw offlineError('私信消息')
  return invoke<DmMessage[]>('x_dm_messages', { conversationId, myId })
}

/** 发送私信。 */
export async function dmSend(conversationId: string, text: string): Promise<void> {
  if (!isTauri()) throw offlineError('发送私信')
  await invoke('x_dm_send', { conversationId, text })
}

/** 关注 / 取关。 */
export async function follow(userId: string, on: boolean): Promise<void> {
  if (!isTauri()) throw offlineError('关注')
  await invoke('x_follow', { userId, on })
}

/** 公开趋势（woeid=1 为全球）。 */
export async function xTrends(woeid = 1): Promise<unknown> {
  if (!isTauri()) throw offlineError('趋势')
  return invoke('x_trends', { woeid })
}

export async function userByScreenName(screenName: string): Promise<XUser> {
  if (!isTauri()) throw offlineError('用户资料')
  return invoke<XUser>('x_user_by_screen_name', { screenName })
}

export async function graphql<T = unknown>(
  operation: string,
  variables: Record<string, unknown>,
  features?: Record<string, unknown>,
): Promise<T> {
  if (!isTauri()) throw offlineError('GraphQL 调用')
  return invoke<T>('x_graphql', { operation, variables, features })
}