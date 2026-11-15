/** X 领域模型，与 Rust 侧 `models.rs` 一一对应（camelCase）。 */

export interface XUser {
  id: string
  name: string
  handle: string
  avatar?: string | null
  banner?: string | null
  bio: string
  verified: boolean
  protected: boolean
  isFollowing: boolean
  followers: number
  following: number
  tweets: number
}

export interface MediaVariant {
  url: string
  contentType: string
  bitrate?: number | null
}

export interface XMedia {
  /** photo | video | animated_gif */
  kind: string
  url: string
  preview?: string | null
  variants: MediaVariant[]
  width: number
  height: number
  altText?: string | null
  durationMs?: number | null
}

export interface XMetrics {
  reply: number
  repost: number
  like: number
  quote: number
  bookmark: number
  view: number
}

export interface XPost {
  id: string
  author: XUser
  text: string
  /** ISO8601 */
  createdAt: string
  lang: string
  metrics: XMetrics
  media: XMedia[]
  isLiked: boolean
  isReposted: boolean
  isBookmarked: boolean
  possiblySensitive: boolean
  quoted?: XPost | null
  reposted?: XPost | null
  noteText?: string | null
  conversationId?: string | null
  inReplyToId?: string | null
  inReplyToHandle?: string | null
}

export interface TimelinePage {
  posts: XPost[]
  topCursor?: string | null
  bottomCursor?: string | null
}

export type TimelineKind = 'forYou' | 'following'

export type ThemeChoice = 'light' | 'dark' | 'black'

export type TabKey = 'home' | 'search' | 'notifications' | 'messages' | 'me'

/** 把 X 的计数格式化成界面显示用的简短形式。 */
export function compactCount(value: number): string {
  if (!value) return ''
  if (value < 10_000) return String(value)
  if (value < 100_000_000) return `${(value / 10_000).toFixed(1).replace(/\.0$/, '')}万`
  return `${(value / 100_000_000).toFixed(1).replace(/\.0$/, '')}亿`
}

/** 把 ISO8601 转成「刚刚 / 5 分钟 / 3 小时 / 11月13日」。 */
export function relativeTime(iso: string): string {
  const time = Date.parse(iso)
  if (Number.isNaN(time)) return ''
  const diff = Date.now() - time
  if (diff < 0) return '刚刚'
  const min = Math.floor(diff / 60_000)
  if (min < 1) return '刚刚'
  if (min < 60) return `${min} 分钟`
  const hour = Math.floor(min / 60)
  if (hour < 24) return `${hour} 小时`
  const day = Math.floor(hour / 24)
  if (day < 7) return `${day} 天`
  const date = new Date(time)
  return `${date.getMonth() + 1}月${date.getDate()}日`
}
/** 通知。Rust 侧已把 X 的富文本渲染成 message。 */
export interface XNotification {
  id: string
  /** like | repost | follow | mention | reply | quote | list | community | other */
  kind: string
  actors: XUser[]
  post?: XPost | null
  message?: string | null
  createdAt?: string | null
}

/** 用户主页时间线的分类。 */
export type UserTimelineKind = 'posts' | 'replies' | 'media' | 'likes'

/** 私信会话。 */
export interface DmConversation {
  id: string
  /** ONE_TO_ONE | GROUP */
  kind: string
  participants: XUser[]
  lastMessage?: string | null
  lastTime?: string | null
  unread: boolean
  muted: boolean
}

/** 私信消息。 */
export interface DmMessage {
  id: string
  conversationId: string
  senderId: string
  text: string
  /** ISO8601 */
  time: string
  fromMe: boolean
}

/** 趋势条目。 */
export interface XTrend {
  name: string
  query: string
  tweetVolume?: number | null
  url?: string | null
}
