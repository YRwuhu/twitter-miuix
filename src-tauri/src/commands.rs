//! Tauri 命令层。
//!
//! 所有网络请求都在 Rust 侧完成，前端只拿结构化结果。
//! 凭据仅保存在内存（`Mutex<Auth>`），退出即清空。
use std::collections::HashMap;
use std::sync::Mutex;

use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;

use crate::models::{self, TimelinePage, XUser};
use crate::query_ids;
use crate::x_api::{home_features, XClient, XError};

pub struct AppState {
    pub client: XClient,
    pub auth: Mutex<crate::x_api::Auth>,
    /// 用户覆盖的 queryId（优先于内置表）
    pub overrides: Mutex<HashMap<String, String>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            client: XClient::new(),
            auth: Mutex::new(crate::x_api::Auth::default()),
            overrides: Mutex::new(HashMap::new()),
        }
    }

    fn auth_snapshot(&self) -> Result<crate::x_api::Auth, String> {
        Ok(self.auth.lock().map_err(|e| e.to_string())?.clone())
    }

    fn resolve(&self, operation: &str) -> Result<String, String> {
        let overrides = self.overrides.lock().map_err(|e| e.to_string())?;
        query_ids::get(operation, &overrides).ok_or_else(|| {
            to_js_error(XError::new(
                0,
                format!("未知操作 {operation}，请先设置 queryId"),
                "unknown_operation",
            ))
        })
    }
}

#[derive(Debug, Serialize)]
pub struct AuthStatus {
    pub logged_in: bool,
    pub guest: bool,
    /// 已登录时的账号信息（可能为 None，例如 cookie 无效）
    pub user: Option<XUser>,
}

fn to_js_error(error: XError) -> String {
    serde_json::to_string(&error).unwrap_or(error.message)
}

/// 要求已登录，否则给出明确错误（而不是让上层收到 404）。
fn require_login(auth: &crate::x_api::Auth) -> Result<(), String> {
    if auth.is_logged_in() {
        Ok(())
    } else {
        Err(to_js_error(XError::new(
            0,
            "需要登录 X 账号",
            "unauthorized",
        )))
    }
}

/* ---------------- 认证 ---------------- */

#[tauri::command]
pub async fn x_auth_status(state: State<'_, AppState>) -> Result<AuthStatus, String> {
    let auth = state.auth_snapshot()?;
    let logged_in = auth.is_logged_in();
    let user = if logged_in {
        current_user(&state).await.ok()
    } else {
        None
    };
    Ok(AuthStatus {
        logged_in,
        guest: auth.guest_token.is_some(),
        user,
    })
}

/// 用 cookie 登录，并**立即验证** cookie 是否有效。
#[tauri::command]
pub async fn x_login(state: State<'_, AppState>, cookie: String) -> Result<AuthStatus, String> {
    let cookie = cookie.trim().to_string();
    if !cookie.contains("auth_token=") || !cookie.contains("ct0=") {
        return Err(to_js_error(XError::new(
            0,
            "cookie 需要同时包含 auth_token 与 ct0",
            "bad_credentials",
        )));
    }

    let candidate = crate::x_api::Auth {
        cookie: Some(cookie.clone()),
        guest_token: None,
    };

    // 用 UserByScreenName 验证登录态：能拿到数据才算登录成功
    let query_id = {
        let overrides = state.overrides.lock().map_err(|e| e.to_string())?;
        query_ids::get("UserByScreenName", &overrides)
            .ok_or_else(|| "缺少 UserByScreenName 的 queryId".to_string())?
    };
    let probe = state
        .client
        .graphql(
            &candidate,
            &query_id,
            "UserByScreenName",
            json!({ "screen_name": "x", "withSafetyModeUserFields": true }),
            Some(json!({ "hidden_birdwatch_note_request_field_enabled": false })),
        )
        .await
        .map_err(to_js_error)?;

    if probe.get("data").is_none() {
        return Err(to_js_error(XError::new(
            0,
            "cookie 无效或已过期",
            "bad_credentials",
        )));
    }

    *state.auth.lock().map_err(|e| e.to_string())? = candidate;
    let _ = crate::secrets::save(&cookie);

    // 登录成功后立刻取一次自己的资料，供头像与个人页使用
    let user = current_user(&state).await.ok();
    Ok(AuthStatus {
        logged_in: true,
        guest: false,
        user,
    })
}

/// 读取当前登录用户（`ViewerUserQuery`）。
async fn current_user(state: &AppState) -> Result<XUser, String> {
    let auth = state.auth_snapshot()?;
    require_login(&auth)?;
    let query_id = state.resolve("ViewerUserQuery")?;
    let data = state
        .client
        .graphql(
            &auth,
            &query_id,
            "ViewerUserQuery",
            json!({}),
            Some(json!({})),
        )
        .await
        .map_err(to_js_error)?;

    let node = data
        .pointer("/data/viewer/userResult/result")
        .ok_or_else(|| to_js_error(XError::new(200, "无法读取当前用户", "parse")))?;
    models::parse_user(node)
        .ok_or_else(|| to_js_error(XError::new(200, "当前用户资料为空", "parse")))
}

/// 启动时尝试恢复上次保存的登录态（DPAPI 解密）。
#[tauri::command]
pub async fn x_restore_session(state: State<'_, AppState>) -> Result<AuthStatus, String> {
    let Ok(Some(cookie)) = crate::secrets::load() else {
        return Ok(AuthStatus {
            logged_in: false,
            guest: false,
            user: None,
        });
    };
    // 验证 cookie 是否还有效
    let candidate = crate::x_api::Auth {
        cookie: Some(cookie.clone()),
        guest_token: None,
    };
    let query_id = {
        let overrides = state.overrides.lock().map_err(|e| e.to_string())?;
        query_ids::get("UserByScreenName", &overrides).unwrap_or_default()
    };
    let probe = state
        .client
        .graphql(
            &candidate,
            &query_id,
            "UserByScreenName",
            json!({ "screen_name": "x", "withSafetyModeUserFields": true }),
            Some(json!({ "hidden_birdwatch_note_request_field_enabled": false })),
        )
        .await;
    if probe.is_err() || probe.as_ref().is_ok_and(|v| v.get("data").is_none()) {
        crate::secrets::clear();
        return Ok(AuthStatus {
            logged_in: false,
            guest: false,
            user: None,
        });
    }
    *state.auth.lock().map_err(|e| e.to_string())? = candidate;
    let user = current_user(&state).await.ok();
    Ok(AuthStatus {
        logged_in: true,
        guest: false,
        user,
    })
}

/// 强制刷新当前用户资料（登录态变化后调用）。
#[tauri::command]
pub async fn x_current_user(state: State<'_, AppState>) -> Result<XUser, String> {
    current_user(&state).await
}

#[tauri::command]
pub async fn x_logout(state: State<'_, AppState>) -> Result<AuthStatus, String> {
    crate::secrets::clear();
    *state.auth.lock().map_err(|e| e.to_string())? = crate::x_api::Auth::default();
    Ok(AuthStatus {
        logged_in: false,
        guest: false,
        user: None,
    })
}

#[tauri::command]
pub async fn x_activate_guest(state: State<'_, AppState>) -> Result<AuthStatus, String> {
    let token = state.client.activate_guest().await.map_err(to_js_error)?;
    let mut auth = state.auth.lock().map_err(|e| e.to_string())?;
    if !auth.is_logged_in() {
        auth.guest_token = Some(token);
    }
    Ok(AuthStatus {
        logged_in: auth.is_logged_in(),
        guest: auth.guest_token.is_some(),
        user: None,
    })
}

/* ---------------- queryId 管理 ---------------- */

#[tauri::command]
pub async fn x_set_query_id(
    state: State<'_, AppState>,
    operation: String,
    query_id: String,
) -> Result<(), String> {
    state
        .overrides
        .lock()
        .map_err(|e| e.to_string())?
        .insert(operation, query_id);
    Ok(())
}

#[tauri::command]
pub async fn x_query_ids(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    let overrides = state.overrides.lock().map_err(|e| e.to_string())?.clone();
    let mut map = query_ids::all();
    map.extend(overrides);
    Ok(map)
}

/* ---------------- 时间线 ---------------- */

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TimelineKind {
    /// 为你推荐（`HomeTimeline`）
    ForYou,
    /// 正在关注（`HomeTimelineLatest`）
    Following,
}

/// 拉取首页时间线。`cursor` 为空表示首屏。
#[tauri::command]
pub async fn x_home_timeline(
    state: State<'_, AppState>,
    kind: TimelineKind,
    cursor: Option<String>,
    count: Option<u32>,
) -> Result<TimelinePage, String> {
    let auth = state.auth_snapshot()?;
    require_login(&auth)?;

    let (operation, mut variables) = match kind {
        TimelineKind::ForYou => (
            "HomeTimeline",
            json!({
                "count": count.unwrap_or(20),
                "includePromotedContent": false,
                "latestControlAvailable": true,
                "requestContext": "launch",
                "withCommunity": true,
                "seenTweetIds": [],
                "autoplay_enabled": false
            }),
        ),
        TimelineKind::Following => (
            "HomeTimelineLatest",
            json!({
                "count": count.unwrap_or(20),
                "includePromotedContent": false,
                "latestControlAvailable": true,
                "withCommunity": true,
                "seenTweetIds": [],
                "autoplay_enabled": false
            }),
        ),
    };
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        variables["cursor"] = Value::String(cursor);
    }

    let query_id = state.resolve(operation)?;
    let data = state
        .client
        .timeline(&auth, &query_id, operation, variables, home_features())
        .await
        .map_err(to_js_error)?;

    Ok(models::parse_timeline(&data))
}

/// 公开趋势（游客可用）。
#[tauri::command]
pub async fn x_trends(state: State<'_, AppState>, woeid: Option<u64>) -> Result<Value, String> {
    let auth = state.auth_snapshot()?;
    let auth = if auth.is_logged_in() || auth.guest_token.is_some() {
        auth
    } else {
        let token = state.client.activate_guest().await.map_err(to_js_error)?;
        crate::x_api::Auth {
            cookie: None,
            guest_token: Some(token),
        }
    };
    state
        .client
        .trends(&auth, woeid.unwrap_or(1))
        .await
        .map_err(to_js_error)
}

/* ---------------- 用户与帖子 ---------------- */

#[tauri::command]
pub async fn x_user_by_screen_name(
    state: State<'_, AppState>,
    screen_name: String,
) -> Result<XUser, String> {
    let auth = state.auth_snapshot()?;
    if !auth.is_logged_in() && auth.guest_token.is_none() {
        return Err(to_js_error(XError::new(
            0,
            "需要登录或游客身份",
            "unauthorized",
        )));
    }
    let query_id = state.resolve("UserByScreenName")?;
    let data = state
        .client
        .graphql(
            &auth,
            &query_id,
            "UserByScreenName",
            json!({ "screen_name": screen_name, "withSafetyModeUserFields": true }),
            Some(json!({ "hidden_birdwatch_note_request_field_enabled": false })),
        )
        .await
        .map_err(to_js_error)?;

    models::parse_user_by_screen_name(&data)
        .ok_or_else(|| to_js_error(XError::new(200, "用户不存在或已被冻结", "not_found")))
}

/// 点赞 / 取消点赞。
#[tauri::command]
pub async fn x_favorite(
    state: State<'_, AppState>,
    tweet_id: String,
    on: bool,
) -> Result<Value, String> {
    let auth = state.auth_snapshot()?;
    require_login(&auth)?;
    let operation = if on {
        "FavoriteTweet"
    } else {
        "UnfavoriteTweet"
    };
    let query_id = state.resolve(operation)?;
    state
        .client
        .graphql_post(
            &auth,
            &query_id,
            operation,
            json!({ "tweet_id": tweet_id }),
            Some(json!({})),
        )
        .await
        .map_err(to_js_error)
}

/// 转推 / 取消转推。
#[tauri::command]
pub async fn x_retweet(
    state: State<'_, AppState>,
    tweet_id: String,
    on: bool,
) -> Result<Value, String> {
    let auth = state.auth_snapshot()?;
    require_login(&auth)?;
    let operation = if on { "CreateRetweet" } else { "DeleteRetweet" };
    let query_id = state.resolve(operation)?;
    state
        .client
        .graphql_post(
            &auth,
            &query_id,
            operation,
            json!({ "tweet_id": tweet_id }),
            Some(json!({})),
        )
        .await
        .map_err(to_js_error)
}

/// 加入 / 移出书签。
#[tauri::command]
pub async fn x_bookmark(
    state: State<'_, AppState>,
    tweet_id: String,
    on: bool,
) -> Result<Value, String> {
    let auth = state.auth_snapshot()?;
    require_login(&auth)?;
    let operation = if on { "BookmarkAdd" } else { "BookmarkDelete" };
    let query_id = state.resolve(operation)?;
    state
        .client
        .graphql_post(
            &auth,
            &query_id,
            operation,
            json!({ "tweet_id": tweet_id }),
            Some(json!({})),
        )
        .await
        .map_err(to_js_error)
}

#[tauri::command]
pub async fn x_upload_photo(
    state: State<'_, AppState>,
    media_base64: String,
    media_type: String,
) -> Result<String, String> {
    let auth = state.auth_snapshot()?;
    require_login(&auth)?;
    if media_base64.len() > 7 * 1024 * 1024 {
        return Err(to_js_error(XError::new(
            0,
            "图片大小不能超过 5 MiB",
            "bad_variables",
        )));
    }
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, media_base64)
        .map_err(|_| to_js_error(XError::new(0, "图片数据无效", "bad_variables")))?;
    validate_photo(&media_type, &bytes)?;
    state
        .client
        .upload_photo(&auth, &bytes, &media_type)
        .await
        .map_err(to_js_error)
}

fn validate_photo(media_type: &str, bytes: &[u8]) -> Result<(), String> {
    let valid = match media_type {
        "image/jpeg" => bytes.starts_with(&[0xff, 0xd8, 0xff]),
        "image/png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "image/gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        _ => false,
    };
    if bytes.is_empty() || bytes.len() > 5 * 1024 * 1024 || !valid {
        return Err(to_js_error(XError::new(
            0,
            "图片格式无效，或大小超过 5 MiB",
            "bad_variables",
        )));
    }
    Ok(())
}

/// 发布帖子或回复。
#[tauri::command]
pub async fn x_create_tweet(
    state: State<'_, AppState>,
    text: String,
    reply_to: Option<String>,
    media_ids: Option<Vec<String>>,
) -> Result<Value, String> {
    let auth = state.auth_snapshot()?;
    require_login(&auth)?;
    let media_ids = media_ids.unwrap_or_default();
    if text.trim().is_empty() && media_ids.is_empty() {
        return Err(to_js_error(XError::new(0, "内容不能为空", "bad_variables")));
    }
    let mut variables = json!({
        "tweet_text": text,
        "media": {
            "media_entities": media_ids.into_iter().map(|media_id| json!({ "media_id": media_id, "tagged_users": [] })).collect::<Vec<_>>(),
            "possibly_sensitive": false
        },
        "semantic_annotation_ids": [],
        "dark_request": false,
    });
    if let Some(reply) = reply_to.filter(|s| !s.is_empty()) {
        variables["reply"] = json!({ "in_reply_to_tweet_id": reply, "exclude_reply_user_ids": [] });
    }
    let query_id = state.resolve("CreateTweet")?;
    state
        .client
        .graphql_post(&auth, &query_id, "CreateTweet", variables, Some(json!({})))
        .await
        .map_err(to_js_error)
}

/* ---------------- 更多时间线 ---------------- */

/// 用户主页时间线（帖子 / 回复 / 媒体 / 喜欢）。
#[tauri::command]
pub async fn x_user_timeline(
    state: State<'_, AppState>,
    user_id: String,
    kind: String,
    cursor: Option<String>,
    count: Option<u32>,
) -> Result<TimelinePage, String> {
    let auth = state.auth_snapshot()?;
    require_login(&auth)?;

    let (operation, mut variables) = match kind.as_str() {
        "replies" => (
            "UserProfileRepliesTimelineQuery",
            json!({
                "rest_id": user_id,
                "count": count.unwrap_or(20),
                "includePromotedContent": false,
                "withCommunity": true,
            }),
        ),
        "media" => (
            "UserProfileMediaTimelineQuery",
            json!({
                "rest_id": user_id,
                "count": count.unwrap_or(20),
                "includePromotedContent": false,
                "withCommunity": true,
            }),
        ),
        "likes" => (
            "UserProfileFavoritesTimelineQuery",
            json!({
                "rest_id": user_id,
                "count": count.unwrap_or(20),
                "includePromotedContent": false,
                "withCommunity": true,
            }),
        ),
        _ => (
            "UserProfileOriginalsTimelineQuery",
            json!({
                "rest_id": user_id,
                "count": count.unwrap_or(20),
                "includePromotedContent": false,
                "withCommunity": true,
            }),
        ),
    };
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        variables["cursor"] = Value::String(cursor);
    }

    let query_id = state.resolve(operation)?;
    let data = state
        .client
        .timeline(&auth, &query_id, operation, variables, home_features())
        .await
        .map_err(to_js_error)?;
    Ok(models::parse_timeline(&data))
}

/// 书签时间线。
#[tauri::command]
pub async fn x_bookmarks(
    state: State<'_, AppState>,
    cursor: Option<String>,
    count: Option<u32>,
) -> Result<TimelinePage, String> {
    let auth = state.auth_snapshot()?;
    require_login(&auth)?;
    let mut variables = json!({
        "count": count.unwrap_or(20),
        "includePromotedContent": false,
        "withCommunity": true,
    });
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        variables["cursor"] = Value::String(cursor);
    }
    let query_id = state.resolve("BookmarksTimelineQuery")?;
    let data = state
        .client
        .timeline(
            &auth,
            &query_id,
            "BookmarksTimelineQuery",
            variables,
            home_features(),
        )
        .await
        .map_err(to_js_error)?;
    Ok(models::parse_timeline(&data))
}

/// 搜索帖子。
#[tauri::command]
pub async fn x_search_timeline(
    state: State<'_, AppState>,
    query: String,
    product: Option<String>,
    cursor: Option<String>,
    count: Option<u32>,
) -> Result<TimelinePage, String> {
    let auth = state.auth_snapshot()?;
    require_login(&auth)?;
    let mut variables = json!({
        "query": query,
        "count": count.unwrap_or(20),
        "query_source": "typed_query",
        "product": product.unwrap_or_else(|| "Latest".into()),
    });
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        variables["cursor"] = Value::String(cursor);
    }
    let query_id = state.resolve("SearchTimeline")?;
    let data = state
        .client
        .timeline(
            &auth,
            &query_id,
            "SearchTimeline",
            variables,
            home_features(),
        )
        .await
        .map_err(to_js_error)?;
    Ok(models::parse_timeline(&data))
}

/// 通知时间线。
#[tauri::command]
pub async fn x_notifications(
    state: State<'_, AppState>,
    user_id: String,
    kind: Option<String>,
    count: Option<u32>,
) -> Result<Vec<models::XNotification>, String> {
    let auth = state.auth_snapshot()?;
    require_login(&auth)?;
    let variables = json!({
        "userId": user_id,
        "count": count.unwrap_or(20),
        "notificationTimelineType": kind.unwrap_or_else(|| "All".into()),
        "includePromotedContent": false,
        "withCommunity": true,
    });
    let query_id = state.resolve("NotificationTimelineQuery")?;
    let data = state
        .client
        .timeline(
            &auth,
            &query_id,
            "NotificationTimelineQuery",
            variables,
            home_features(),
        )
        .await
        .map_err(to_js_error)?;
    Ok(models::parse_notifications(&data))
}

/// 帖子详情（主帖 + 回复）。
#[tauri::command]
pub async fn x_tweet_detail(
    state: State<'_, AppState>,
    tweet_id: String,
) -> Result<TimelinePage, String> {
    let auth = state.auth_snapshot()?;
    require_login(&auth)?;
    let variables = json!({
        "focalTweetId": tweet_id,
        "includePromotedContent": true,
        "withBirdwatchNotes": true,
        "withVoice": true,
        "withCommunity": true,
        "rankingMode": "Relevance",
    });
    let query_id = state.resolve("TweetDetail")?;
    let data = state
        .client
        .timeline(&auth, &query_id, "TweetDetail", variables, home_features())
        .await
        .map_err(to_js_error)?;

    let (main, replies) = models::parse_tweet_detail(&data);
    let mut posts = Vec::new();
    if let Some(main) = main {
        posts.push(main);
    }
    posts.extend(replies);
    Ok(TimelinePage {
        posts,
        top_cursor: None,
        bottom_cursor: None,
    })
}

/* ---------------- 私信（REST） ---------------- */

/// 私信会话列表。
#[tauri::command]
pub async fn x_dm_conversations(
    state: State<'_, AppState>,
    my_id: String,
) -> Result<Vec<crate::dm::DmConversation>, String> {
    let auth = state.auth_snapshot()?;
    require_login(&auth)?;
    let data = state
        .client
        .rest_get(&auth, "/1.1/dm/inbox_initial_state.json", &[])
        .await
        .map_err(to_js_error)?;
    Ok(crate::dm::parse_inbox(&data, &my_id))
}

/// 某个会话的消息列表。
#[tauri::command]
pub async fn x_dm_messages(
    state: State<'_, AppState>,
    conversation_id: String,
    my_id: String,
) -> Result<Vec<crate::dm::DmMessage>, String> {
    let auth = state.auth_snapshot()?;
    require_login(&auth)?;
    let path = format!("/1.1/dm/conversation/{conversation_id}.json");
    let data = state
        .client
        .rest_get(&auth, &path, &[("include_events", "true".into())])
        .await
        .map_err(to_js_error)?;
    Ok(crate::dm::parse_conversation(&data, &my_id))
}

/// 发送私信。
#[tauri::command]
pub async fn x_dm_send(
    state: State<'_, AppState>,
    conversation_id: String,
    text: String,
) -> Result<Value, String> {
    let auth = state.auth_snapshot()?;
    require_login(&auth)?;
    if text.trim().is_empty() {
        return Err(to_js_error(XError::new(0, "内容不能为空", "bad_variables")));
    }
    state
        .client
        .rest_post(
            &auth,
            "/1.1/dm/chat/send.json",
            &[
                ("conversation_id", conversation_id),
                ("text", text),
                (
                    "request_id",
                    format!(
                        "{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_millis())
                            .unwrap_or(0)
                    ),
                ),
            ],
        )
        .await
        .map_err(to_js_error)
}

/* ---------------- 关注 / 屏蔽 ---------------- */

/// 关注或取关。
#[tauri::command]
pub async fn x_follow(
    state: State<'_, AppState>,
    user_id: String,
    on: bool,
) -> Result<Value, String> {
    let auth = state.auth_snapshot()?;
    require_login(&auth)?;
    let path = if on {
        "/1.1/friendships/create.json"
    } else {
        "/1.1/friendships/destroy.json"
    };
    state
        .client
        .rest_post(&auth, path, &[("user_id", user_id)])
        .await
        .map_err(to_js_error)
}

/// 通用 GraphQL 逃生口，便于在「接口」面板里调试任意操作。
#[tauri::command]
pub async fn x_graphql(
    state: State<'_, AppState>,
    operation: String,
    variables: Value,
    features: Option<Value>,
) -> Result<Value, String> {
    let auth = state.auth_snapshot()?;
    let query_id = state.resolve(&operation)?;
    state
        .client
        .graphql(&auth, &query_id, &operation, variables, features)
        .await
        .map_err(to_js_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_login_rejects_empty_auth() {
        let auth = crate::x_api::Auth::default();
        let err = require_login(&auth).expect_err("未登录应报错");
        assert!(err.contains("unauthorized"));
    }

    #[test]
    fn require_login_accepts_cookie() {
        let auth = crate::x_api::Auth {
            cookie: Some("auth_token=a; ct0=b".into()),
            guest_token: None,
        };
        assert!(require_login(&auth).is_ok());
    }

    #[test]
    fn validates_supported_photo_signatures() {
        assert!(validate_photo("image/jpeg", &[0xff, 0xd8, 0xff, 0]).is_ok());
        assert!(validate_photo("image/png", b"\x89PNG\r\n\x1a\n").is_ok());
        assert!(validate_photo("image/gif", b"GIF89a").is_ok());
        assert!(validate_photo("image/png", b"not an image").is_err());
        assert!(validate_photo("image/webp", b"RIFF").is_err());
    }
}
