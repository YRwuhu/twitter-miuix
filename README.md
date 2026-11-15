# twitter-miuix

X/Twitter 第三方桌面客户端，HyperOS / miuix 视觉风格。**已接入真实 X 数据**。

**技术栈**：Rust + Tauri 2 + Vue 3 + TypeScript + Pinia + [miuix-vue](https://github.com/YuKongA/miuix-vue)

## 快速开始

1. 安装 `twitter-miuix_0.1.0_x64-setup.exe`
2. 打开应用 → 点「登录 X」
3. 粘贴 cookie：浏览器登录 x.com → F12 → Application → Cookies → `https://x.com`
   → 复制 `auth_token` 和 `ct0` 的值，拼成 `auth_token=...; ct0=...`
4. 登录后会立刻验证 cookie 并拉取你的时间线

凭据用 Windows DPAPI 加密后保存在 `%APPDATA%/twitter-miuix/session.bin`，
绑定到当前 Windows 用户；下次启动自动恢复登录态。点「退出」会清除。

## 当前实现

### 已接入真实接口

| 功能 | 接口 | 状态 |
|---|---|---|
| 首页「为你推荐」 | `HomeTimeline` | ✅ 真实数据，游标分页 |
| 首页「正在关注」 | `HomeTimelineLatest` | ✅ 真实数据 |
| 当前登录用户 | `ViewerUserQuery` | ✅ 头像/昵称/粉丝数 |
| 用户资料查询 | `UserByScreenName` | ✅ 搜索页可用（游客也行） |
| 点赞 / 取消 | `FavoriteTweet` / `UnfavoriteTweet` | ✅ 乐观更新 + 失败回滚 |
| 转推 / 取消 | `CreateRetweet` / `DeleteRetweet` | ✅ |
| 书签 / 移出 | `BookmarkAdd` / `BookmarkDelete` | ✅ |
| 发帖 / 回复 | `CreateTweet` | ✅ |
| 帖子详情 + 回复列表 | `TweetDetail` | ✅ 实测 36 条回复 |
| 通知流（全部/提及） | `NotificationTimelineQuery` | ✅ 实测可用 |
| 帖子搜索 | `SearchTimeline` | ✅ 实测 20 条 |
| 用户主页（帖子/回复/媒体/喜欢） | `UserProfile*TimelineQuery` | ✅ 实测可用 |
| 书签时间线 | `BookmarksTimelineQuery` | ✅ 实测可用 |
| 公开趋势 | `1.1/trends/place.json` | ✅ 搜索页第三个 tab |
| 私信会话与消息 | `1.1/dm/*` | ✅ 实测 2 会话 / 5 消息 |
| 发送私信 | `1.1/dm/chat/send.json` | ✅ |
| 关注 / 取关 | `1.1/friendships/*` | ✅ 用户主页按钮 |
| 登录态持久化 | Windows DPAPI | ✅ 加密存盘、启动自动恢复 |

### 数据层事实（实测确认）

- 请求形态：`/graphql/{queryId}/{OperationName}`
- **query 用 GET 或 POST 都可以；mutation 必须用 POST**
  （GET 调 mutation 会返回 `406 GET requests only allow query operations`）
- 认证：公开 Bearer + cookie（`auth_token` + `ct0`）
- `x-csrf-token` 取 cookie 里的 `ct0`
- **未登录时接口会返回 404，不是 403**。这是最容易误判的地方：
  之前据此认为「queryId 失效」是错的，实际只是没登录。
- 309 条 queryId 来自静态提取的注册表（`src-tauri/data/gql_ops.json`），
  失效时可在设置 → 接口里覆盖。

### 尚未实现

- XChat 新版端到端加密私信（当前用的是旧版 REST DM，不支持加密会话）
- 群聊管理、消息请求、已读回执
- 媒体上传（发帖只支持纯文本）
- 社区、列表管理、书签集合
- 屏蔽 / 静音、举报、编辑资料、多账号
- 帖子搜索之外的搜索筛选（用户/位置/时间）

## 架构

```
前端 Vue 3
├─ Pinia          stores/timeline.ts（双时间线、游标分页、骨架/错误态）
├─ 视图            Home / Detail / Search / Notifications / Bookmarks /
│                  Messages（含聊天） / Profile / UserProfile
└─ 组件            帖子卡、媒体网格、登录面板、发帖面板

Rust (src-tauri)
├─ x_api.rs       请求层（GET query / POST mutation、认证头、错误映射）
├─ models.rs      X 领域模型 + GraphQL 响应解析
├─ dm.rs          私信（REST）解析
├─ secrets.rs     Windows DPAPI 凭据加密
├─ query_ids.rs   309 条 queryId 注册表（含覆盖机制）
└─ commands.rs    Tauri 命令（25 个）
```

## 开发

```bash
cd desktop
npm install
npm run tauri:dev     # 开发
npm run tauri:build   # 打包

cd src-tauri
cargo test            # 单元测试（16 个，含 DPAPI 往返，不联网）
X_COOKIE="auth_token=...; ct0=..." cargo test -- --ignored --nocapture   # 真实外网集成测试
```

Windows 需要 MSVC 链接器（Visual Studio Build Tools）与 Windows SDK。

## 视觉

HyperOS 风格：**平面列表 + 大标题**，不再用「内容塞进卡片」。

- 时间线、详情、通知是平面列表，靠 1px 分隔线与留白组织，不用卡片包裹
- 页面标题 30px / 700，与 HyperOS 的首页大标题对齐
- 底栏选中项带柔和胶囊底，图标缩放反馈
- 空态居中留白 + 圆角图标块，不用卡片
- 卡片仅用于设置分组这类真正的容器场景

### 字体

`miuix-vue` 本身**不打包任何字体**（样式里全是 `font-family: inherit`），
也不含 MiSans。本项目用双字体栈：

使用 **MiSans**（小米官方字体，Apache-2.0），即 HyperOS 的系统字体。

来源：<https://hyperos.mi.com/font/> · 许可：Apache License 2.0

按界面实际用到的 573 个字符做了子集，两个字重合计 **117KB**
（完整 MiSans 单个字重约 10MB）。拉丁与中文统一用 MiSans，观感与 HyperOS 一致。

## 窗口

整个窗口固定为手机竖屏比例 9:19.5（基准 432×936），按屏幕等比缩放。
Rust 侧显式声明 Per-Monitor V2 DPI 感知，避免高 DPI 下 WebView2 与窗口缩放不一致导致裁切。

## 关于逆向

接口细节见 `src-tauri/data/gql_ops.json`，含 309 个操作的 queryId 与认证方式。

⚠️ 该方案依赖 X 的内部 GraphQL 接口与真实账号 cookie，属于非官方途径。
queryId 会随 X 版本轮换，届时需要在设置里更新。