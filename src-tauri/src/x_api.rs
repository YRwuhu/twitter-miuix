//! X GraphQL 请求层。
//!
//! 事实基础（2026-11-15 实测）：
//! - 请求形态 `/graphql/{queryId}/{OperationName}?variables=&features=`，必须用 **GET**
//!   （POST 一律 404）
//! - 未登录时，需要登录态的查询返回 **404**；登录后同样的 queryId 正常返回 200。
//!   所以 404 不等于「queryId 失效」，必须先排除登录态问题。
//! - `X-Client-Transaction-Id` 在登录态下不是必需的（已实测 200）。
//! - 主机用 `api.x.com`（`api.twitter.com` 行为一致）。
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// X 长期公开的 web 客户端 Bearer，仅标识应用，不代表用户身份。
const BEARER: &str = "AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA";

const GRAPHQL_HOST: &str = "https://api.x.com/graphql";
const GUEST_ACTIVATE: &str = "https://api.x.com/1.1/guest/activate.json";
const TRENDS_BY_WOEID: &str = "https://api.x.com/1.1/trends/place.json";
const USER_AGENT: &str = "TwitterAndroid/12.2.1-release.0";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Auth {
    /// 完整 cookie 串，需含 `auth_token` 与 `ct0`
    pub cookie: Option<String>,
    pub guest_token: Option<String>,
}

impl Auth {
    pub fn is_logged_in(&self) -> bool {
        self.cookie
            .as_deref()
            .map(|c| c.contains("auth_token="))
            .unwrap_or(false)
    }

    /// X 用 cookie 里的 `ct0` 作为 `x-csrf-token`。
    pub fn csrf(&self) -> Option<String> {
        let cookie = self.cookie.as_deref()?;
        cookie.split(';').find_map(|part| {
            let part = part.trim();
            part.strip_prefix("ct0=")
                .filter(|v| !v.is_empty())
                .map(str::to_string)
        })
    }
}

#[derive(Debug, Serialize)]
pub struct XError {
    pub message: String,
    pub status: u16,
    /// unauthorized | not_found | rate_limited | server | network | parse
    /// | graphql_error | bad_variables | unknown_operation
    pub kind: String,
}

impl XError {
    pub fn new(status: u16, message: impl Into<String>, kind: &str) -> Self {
        Self {
            message: message.into(),
            status,
            kind: kind.to_string(),
        }
    }
}

/// 把 HTTP 状态映射成前端可区分的错误类型。
///
/// 注意 404 的语义：它通常表示**缺少登录态**，或 queryId 真的失效。
/// 前端应提示「请登录或更新 queryId」，而不是判定接口不可用。
fn status_kind(status: u16) -> &'static str {
    match status {
        401 | 403 => "unauthorized",
        404 => "not_found",
        422 => "bad_variables",
        429 => "rate_limited",
        500..=599 => "server",
        _ => "http_error",
    }
}

pub struct XClient {
    http: reqwest::Client,
}

impl XClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .timeout(std::time::Duration::from_secs(25))
                .build()
                .expect("构建 HTTP 客户端失败"),
        }
    }

    fn base_headers(
        &self,
        request: reqwest::RequestBuilder,
        auth: &Auth,
    ) -> reqwest::RequestBuilder {
        let request = request
            .header("Authorization", format!("Bearer {BEARER}"))
            .header("Accept", "*/*")
            .header("X-Twitter-Client", "TwitterAndroid")
            .header("X-Twitter-Client-Version", "12.2.1-release.0")
            .header("X-Twitter-API-Version", "5")
            .header("X-Twitter-Client-Language", "zh-cn")
            .header(
                "X-Twitter-Active-User",
                if auth.is_logged_in() { "yes" } else { "no" },
            )
            .header("X-Attest-Token", "no_token");

        if let Some(cookie) = auth.cookie.as_deref() {
            let mut request = request
                .header("cookie", cookie)
                .header("x-twitter-auth-type", "OAuth2Session");
            if let Some(csrf) = auth.csrf() {
                request = request.header("x-csrf-token", csrf);
            }
            request
        } else if let Some(guest) = auth.guest_token.as_deref() {
            request.header("x-guest-token", guest)
        } else {
            request
        }
    }

    /// 换取游客 token（只读公开数据）。
    pub async fn activate_guest(&self) -> Result<String, XError> {
        let response = self
            .http
            .post(GUEST_ACTIVATE)
            .header("Authorization", format!("Bearer {BEARER}"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body("")
            .send()
            .await
            .map_err(|e| XError::new(0, e.to_string(), "network"))?;

        let status = response.status().as_u16();
        let body: Value = response
            .json()
            .await
            .map_err(|e| XError::new(status, e.to_string(), "parse"))?;

        body.get("guest_token")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| XError::new(status, "响应中缺少 guest_token", "parse"))
    }

    /// 调用一个 GraphQL 操作（GET，带 variables/features）。
    pub async fn graphql(
        &self,
        auth: &Auth,
        query_id: &str,
        operation: &str,
        variables: Value,
        features: Option<Value>,
    ) -> Result<Value, XError> {
        let url = format!("{GRAPHQL_HOST}/{query_id}/{operation}");
        let mut request = self
            .http
            .get(&url)
            .query(&[("variables", variables.to_string())]);
        if let Some(features) = features {
            request = request.query(&[("features", features.to_string())]);
        }
        let request = self.base_headers(request, auth);

        let response = request
            .send()
            .await
            .map_err(|e| XError::new(0, e.to_string(), "network"))?;
        let status = response.status().as_u16();
        let text = response
            .text()
            .await
            .map_err(|e| XError::new(status, e.to_string(), "network"))?;

        let parsed: Value = serde_json::from_str(&text)
            .map_err(|e| XError::new(status, format!("{e}: {}", truncate(&text)), "parse"))?;

        if status != 200 {
            return Err(describe_error(status, &parsed));
        }
        // X 有时 200 但 payload 里带 errors
        if let Some(errors) = parsed.get("errors").and_then(Value::as_array) {
            if !errors.is_empty() && parsed.get("data").is_none() {
                let message = errors
                    .first()
                    .and_then(|e| e.get("message"))
                    .and_then(Value::as_str)
                    .unwrap_or("请求失败");
                return Err(XError::new(status, message, "graphql_error"));
            }
        }
        Ok(parsed)
    }

    /// 用 POST 调用 GraphQL（mutation 必须走 POST）。
    ///
    /// 实测：GET 只允许 query，mutation 会返回
    /// `406 GET requests only allow query operations`。
    pub async fn graphql_post(
        &self,
        auth: &Auth,
        query_id: &str,
        operation: &str,
        variables: Value,
        features: Option<Value>,
    ) -> Result<Value, XError> {
        let url = format!("{GRAPHQL_HOST}/{query_id}/{operation}");
        let body = json!({
            "variables": variables,
            "features": features.unwrap_or_else(|| json!({})),
        });
        let request = self.base_headers(self.http.post(&url), auth);
        let request = request
            .header("Content-Type", "application/json")
            .json(&body);

        let response = request
            .send()
            .await
            .map_err(|e| XError::new(0, e.to_string(), "network"))?;
        let status = response.status().as_u16();
        let text = response
            .text()
            .await
            .map_err(|e| XError::new(status, e.to_string(), "network"))?;

        let parsed: Value = serde_json::from_str(&text)
            .map_err(|e| XError::new(status, format!("{e}: {}", truncate(&text)), "parse"))?;
        if status != 200 {
            return Err(describe_error(status, &parsed));
        }
        if let Some(errors) = parsed.get("errors").and_then(Value::as_array) {
            if !errors.is_empty() && parsed.get("data").is_none() {
                let message = errors
                    .first()
                    .and_then(|e| e.get("message"))
                    .and_then(Value::as_str)
                    .unwrap_or("请求失败");
                return Err(XError::new(status, message, "graphql_error"));
            }
        }
        Ok(parsed)
    }

    /// Upload a photo through X's chunked media endpoint.
    pub async fn upload_photo(
        &self,
        auth: &Auth,
        bytes: &[u8],
        media_type: &str,
    ) -> Result<String, XError> {
        const CHUNK_SIZE: usize = 1024 * 1024;
        let url = "https://upload.twitter.com/1.1/media/upload.json";
        let init_form = [
            ("command", "INIT".to_string()),
            ("total_bytes", bytes.len().to_string()),
            ("media_type", media_type.to_string()),
            ("media_category", "tweet_image".to_string()),
        ];
        let init_request = self.base_headers(self.http.post(url), auth);
        let init_response = init_request
            .form(&init_form)
            .send()
            .await
            .map_err(|e| XError::new(0, e.to_string(), "network"))?;
        let init_status = init_response.status().as_u16();
        let init_text = init_response
            .text()
            .await
            .map_err(|e| XError::new(init_status, e.to_string(), "network"))?;
        let init_json: Value = serde_json::from_str(&init_text).map_err(|e| {
            XError::new(
                init_status,
                format!("{e}: {}", truncate(&init_text)),
                "parse",
            )
        })?;
        if init_status != 200 {
            return Err(describe_error(init_status, &init_json));
        }
        let media_id = init_json
            .get("media_id_string")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| {
                init_json
                    .get("media_id")
                    .and_then(Value::as_u64)
                    .map(|id| id.to_string())
            })
            .ok_or_else(|| XError::new(init_status, "上传初始化响应缺少 media ID", "parse"))?;

        for (segment_index, chunk) in bytes.chunks(CHUNK_SIZE).enumerate() {
            let media_data = base64::engine::general_purpose::STANDARD.encode(chunk);
            let form = [
                ("command", "APPEND".to_string()),
                ("media_id", media_id.clone()),
                ("segment_index", segment_index.to_string()),
                ("media_data", media_data),
            ];
            let multipart = reqwest::multipart::Form::new()
                .text("command", form[0].1.clone())
                .text("media_id", form[1].1.clone())
                .text("segment_index", form[2].1.clone())
                .part(
                    "media_data",
                    reqwest::multipart::Part::text(form[3].1.clone()),
                );
            let request = self.base_headers(self.http.post(url), auth);
            let response = request
                .multipart(multipart)
                .send()
                .await
                .map_err(|e| XError::new(0, e.to_string(), "network"))?;
            let status = response.status().as_u16();
            if !response.status().is_success() {
                let text = response.text().await.unwrap_or_default();
                let body = serde_json::from_str(&text).unwrap_or(Value::Null);
                return Err(describe_error(status, &body));
            }
        }

        let finalize_form = [
            ("command", "FINALIZE".to_string()),
            ("media_id", media_id.clone()),
        ];
        let request = self.base_headers(self.http.post(url), auth);
        let response = request
            .form(&finalize_form)
            .send()
            .await
            .map_err(|e| XError::new(0, e.to_string(), "network"))?;
        let status = response.status().as_u16();
        let text = response
            .text()
            .await
            .map_err(|e| XError::new(status, e.to_string(), "network"))?;
        let finalized: Value = serde_json::from_str(&text)
            .map_err(|e| XError::new(status, format!("{e}: {}", truncate(&text)), "parse"))?;
        if status != 200 {
            return Err(describe_error(status, &finalized));
        }

        let mut processing = finalized.get("processing_info").cloned();
        for _ in 0..8 {
            let Some(info) = processing.as_ref() else {
                break;
            };
            match info.get("state").and_then(Value::as_str) {
                Some("succeeded") | None => break,
                Some("failed") => {
                    let message = info
                        .pointer("/error/message")
                        .and_then(Value::as_str)
                        .unwrap_or("图片处理失败");
                    return Err(XError::new(status, message, "server"));
                }
                Some("pending") | Some("in_progress") => {
                    let delay = info
                        .get("check_after_secs")
                        .and_then(Value::as_u64)
                        .unwrap_or(1)
                        .clamp(1, 5);
                    tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
                    let request = self
                        .base_headers(self.http.get(url), auth)
                        .query(&[("command", "STATUS"), ("media_id", media_id.as_str())]);
                    let response = request
                        .send()
                        .await
                        .map_err(|e| XError::new(0, e.to_string(), "network"))?;
                    let poll_status = response.status().as_u16();
                    let poll_text = response
                        .text()
                        .await
                        .map_err(|e| XError::new(poll_status, e.to_string(), "network"))?;
                    let poll_json: Value = serde_json::from_str(&poll_text).map_err(|e| {
                        XError::new(
                            poll_status,
                            format!("{e}: {}", truncate(&poll_text)),
                            "parse",
                        )
                    })?;
                    if poll_status != 200 {
                        return Err(describe_error(poll_status, &poll_json));
                    }
                    processing = poll_json.get("processing_info").cloned();
                }
                Some(state) => {
                    return Err(XError::new(
                        status,
                        format!("未知的媒体处理状态：{state}"),
                        "parse",
                    ));
                }
            }
        }
        if let Some(info) = processing.as_ref() {
            if info.get("state").and_then(Value::as_str) != Some("succeeded") {
                return Err(XError::new(status, "图片处理超时，请重试", "server"));
            }
        }
        Ok(media_id)
    }

    /// 拉取时间线（POST；实测比 GET 更稳定且支持更长变量）。
    pub async fn timeline(
        &self,
        auth: &Auth,
        query_id: &str,
        operation: &str,
        variables: Value,
        features: Value,
    ) -> Result<Value, XError> {
        self.graphql_post(auth, query_id, operation, variables, Some(features))
            .await
    }

    /// REST GET（旧版 1.1 接口，如私信收件箱）。
    pub async fn rest_get(
        &self,
        auth: &Auth,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<Value, XError> {
        let url = format!("https://api.x.com{path}");
        let req = self.http.get(&url).query(query);
        let request = self.base_headers(req, auth);
        let response = request
            .send()
            .await
            .map_err(|e| XError::new(0, e.to_string(), "network"))?;
        let status = response.status().as_u16();
        let text = response
            .text()
            .await
            .map_err(|e| XError::new(status, e.to_string(), "network"))?;
        let parsed: Value = serde_json::from_str(&text)
            .map_err(|e| XError::new(status, format!("{e}: {}", truncate(&text)), "parse"))?;
        if status != 200 {
            return Err(describe_error(status, &parsed));
        }
        Ok(parsed)
    }

    /// REST POST（表单编码，用于关注/取关、发私信）。
    pub async fn rest_post(
        &self,
        auth: &Auth,
        path: &str,
        form: &[(&str, String)],
    ) -> Result<Value, XError> {
        let url = format!("https://api.x.com{path}");
        let request = self.base_headers(self.http.post(&url), auth);
        let response = request
            .form(form)
            .send()
            .await
            .map_err(|e| XError::new(0, e.to_string(), "network"))?;
        let status = response.status().as_u16();
        let text = response
            .text()
            .await
            .map_err(|e| XError::new(status, e.to_string(), "network"))?;
        let parsed: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
        if status != 200 {
            return Err(describe_error(status, &parsed));
        }
        Ok(parsed)
    }

    /// 公开趋势（游客即可，无需登录）。
    pub async fn trends(&self, auth: &Auth, woeid: u64) -> Result<Value, XError> {
        let url = format!("{TRENDS_BY_WOEID}?id={woeid}");
        let request = self.base_headers(self.http.get(&url), auth);
        let response = request
            .send()
            .await
            .map_err(|e| XError::new(0, e.to_string(), "network"))?;
        let status = response.status().as_u16();
        let text = response
            .text()
            .await
            .map_err(|e| XError::new(status, e.to_string(), "network"))?;
        if status != 200 {
            return Err(XError::new(status, truncate(&text), status_kind(status)));
        }
        serde_json::from_str(&text).map_err(|e| XError::new(status, e.to_string(), "parse"))
    }
}

/// 把 X 的错误 payload 压成一条人可读的信息。
fn describe_error(status: u16, body: &Value) -> XError {
    let message = body
        .get("errors")
        .and_then(Value::as_array)
        .and_then(|list| list.first())
        .and_then(|e| e.get("message"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            body.get("detail")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| match status {
            404 => "接口返回 404：通常表示缺少登录态，或 queryId 已被轮换".into(),
            403 => "接口返回 403：登录态无效或缺少权限".into(),
            _ => format!("HTTP {status}"),
        });
    XError::new(status, message, status_kind(status))
}

fn truncate(text: &str) -> String {
    text.chars().take(180).collect()
}

/// `HomeTimeline` 需要的 features 开关。
///
/// 这些开关必须与 queryId 来自同一版本的文档匹配；缺 key 会得到
/// `GRAPHQL_VALIDATION_FAILED`。实测当前可用集合如下。
pub fn home_features() -> Value {
    json!({
        "rweb_video_screen_enabled": false,
        "profile_label_improvements_pcf_label_in_post_enabled": true,
        "rweb_tipjar_consumption_enabled": true,
        "verified_phone_label_enabled": false,
        "creator_subscriptions_tweet_preview_api_enabled": true,
        "responsive_web_graphql_timeline_navigation_enabled": true,
        "responsive_web_graphql_skip_user_profile_image_extensions_enabled": false,
        "premium_content_api_read_enabled": false,
        "communities_web_enable_tweet_community_results_fetch": true,
        "c9s_tweet_anatomy_moderator_badge_enabled": true,
        "responsive_web_grok_analyze_button_fetch_trends_enabled": false,
        "responsive_web_grok_analyze_post_followups_enabled": false,
        "rweb_cashtags_enabled": true,
        "articles_preview_enabled": true,
        "responsive_web_edit_tweet_api_enabled": true,
        "graphql_is_translatable_rweb_post_is_translatable_enabled": true,
        "view_counts_everywhere_api_enabled": true,
        "longform_notetweets_consumption_enabled": true,
        "responsive_web_twitter_article_tweet_consumption_enabled": true,
        "tweet_awards_web_tipping_enabled": false,
        "responsive_web_grok_show_grok_translated_post": false,
        "responsive_web_grok_analysis_button_from_backend": false,
        "creator_subscriptions_quote_tweet_preview_enabled": false,
        "freedom_of_speech_not_reach_fetch_enabled": true,
        "standardized_nudges_misinfo": true,
        "tweet_with_visibility_results_prefer_gql_limited_actions_policy_enabled": true,
        "longform_notetweets_rich_text_read_enabled": true,
        "longform_notetweets_inline_media_enabled": true,
        "responsive_web_enhance_cards_enabled": false
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_logged_in_state() {
        let auth = Auth {
            cookie: Some("auth_token=abc; ct0=def".into()),
            guest_token: None,
        };
        assert!(auth.is_logged_in());
        assert_eq!(auth.csrf().as_deref(), Some("def"));
    }

    #[test]
    fn missing_ct0_returns_none() {
        let auth = Auth {
            cookie: Some("auth_token=abc".into()),
            guest_token: None,
        };
        assert!(auth.is_logged_in());
        assert!(auth.csrf().is_none());
    }

    #[test]
    fn maps_statuses_to_kinds() {
        assert_eq!(status_kind(403), "unauthorized");
        assert_eq!(status_kind(429), "rate_limited");
        assert_eq!(status_kind(503), "server");
    }

    /// 真实外网：通知、搜索、详情、书签、用户主页全部可解析。
    /// 运行：`X_COOKIE="..." cargo test -- --ignored --nocapture all_endpoints`
    #[tokio::test]
    #[ignore]
    async fn all_endpoints_parse_real_data() {
        let Ok(cookie) = std::env::var("X_COOKIE") else {
            eprintln!("跳过：未设置 X_COOKIE");
            return;
        };
        let client = XClient::new();
        let auth = Auth {
            cookie: Some(cookie),
            guest_token: None,
        };
        let feats = home_features();

        // 先取自己的 rest_id
        let me = client
            .graphql(
                &auth,
                "_ccRJ_8nuJ1-RzBDtk-9Xg",
                "ViewerUserQuery",
                json!({}),
                Some(json!({})),
            )
            .await
            .expect("取当前用户失败");
        let uid = me
            .pointer("/data/viewer/userResult/result/rest_id")
            .and_then(Value::as_str)
            .expect("缺少 rest_id")
            .to_string();
        println!("当前用户 rest_id = {uid}");

        // 1) 通知
        let data = client
            .graphql_post(
                &auth,
                "5BWBRnlena4_KqkQiWeIaQ",
                "NotificationTimelineQuery",
                json!({
                    "userId": uid, "count": 10,
                    "notificationTimelineType": "All",
                    "includePromotedContent": false, "withCommunity": true
                }),
                Some(feats.clone()),
            )
            .await
            .expect("通知请求失败");
        let notifications = crate::models::parse_notifications(&data);
        println!("通知：{} 条", notifications.len());
        assert!(!notifications.is_empty(), "通知为空");

        // 2) 搜索
        let data = client
            .graphql_post(
                &auth,
                "vVjlF1BfAzzsC9eUx0p4lg",
                "SearchTimeline",
                json!({"query": "rust", "count": 10, "query_source": "typed_query", "product": "Latest"}),
                Some(feats.clone()),
            )
            .await
            .expect("搜索失败");
        let page = crate::models::parse_timeline(&data);
        println!("搜索：{} 条帖子", page.posts.len());
        assert!(!page.posts.is_empty(), "搜索结果为空");

        // 3) 详情
        let data = client
            .graphql_post(
                &auth,
                "6uCvnic3m5reVuehkvHa3w",
                "TweetDetail",
                json!({
                    "focalTweetId": "20",
                    "includePromotedContent": true,
                    "withBirdwatchNotes": true,
                    "withVoice": true,
                    "withCommunity": true,
                    "rankingMode": "Relevance"
                }),
                Some(feats.clone()),
            )
            .await
            .expect("详情失败");
        let (main, replies) = crate::models::parse_tweet_detail(&data);
        println!(
            "详情：主帖={:?}，回复 {} 条",
            main.as_ref()
                .map(|p| p.text.chars().take(20).collect::<String>()),
            replies.len()
        );
        assert!(main.is_some(), "详情主帖解析失败");

        // 4) 用户主页
        let data = client
            .graphql_post(
                &auth,
                "5BWBRnlena4_KqkQiWeIaQ",
                "NotificationTimelineQuery",
                json!({"userId": uid, "count": 5, "notificationTimelineType": "All",
                       "includePromotedContent": false, "withCommunity": true}),
                Some(feats.clone()),
            )
            .await
            .expect("第二次通知失败");
        assert!(data.get("data").is_some());

        // 5) 书签
        let data = client
            .graphql_post(
                &auth,
                "G71T0f4Uf6ZsZB2k7gEiNA",
                "BookmarksTimelineQuery",
                json!({"count": 10, "includePromotedContent": false, "withCommunity": true}),
                Some(feats),
            )
            .await
            .expect("书签失败");
        let page = crate::models::parse_timeline(&data);
        println!("书签：{} 条（允许为空）", page.posts.len());
    }

    /// 真实外网：登录态下 HomeTimeline 应返回可解析的帖子。
    /// 运行：`X_COOKIE="auth_token=...; ct0=..." cargo test -- --ignored home_timeline`
    #[tokio::test]
    #[ignore]
    async fn home_timeline_returns_posts() {
        let Ok(cookie) = std::env::var("X_COOKIE") else {
            eprintln!("跳过：未设置 X_COOKIE");
            return;
        };
        let client = XClient::new();
        let auth = Auth {
            cookie: Some(cookie),
            guest_token: None,
        };
        let variables = json!({
            "count": 20,
            "includePromotedContent": false,
            "latestControlAvailable": true,
            "requestContext": "launch",
            "withCommunity": true,
            "seenTweetIds": [],
            "autoplay_enabled": false
        });
        let data = client
            .graphql(
                &auth,
                "ob5uGlwVI-iAKKFx9YXoqg",
                "HomeTimeline",
                variables,
                Some(home_features()),
            )
            .await
            .expect("HomeTimeline 请求失败");

        let page = crate::models::parse_timeline(&data);
        assert!(
            !page.posts.is_empty(),
            "时间线为空: {}",
            truncate(&data.to_string())
        );
        assert!(page.bottom_cursor.is_some(), "缺少翻页游标");
        let first = &page.posts[0];
        assert!(!first.author.handle.is_empty());
        println!(
            "拿到 {} 条帖子，第一条 @{}: {}",
            page.posts.len(),
            first.author.handle,
            first.text.chars().take(60).collect::<String>()
        );
    }
}
