//! X 领域模型与 GraphQL 响应解析。
//!
//! 字段路径来自实测响应（2026-11-15，`HomeTimeline` 真实返回）：
//! - 帖子：`entry.content.content.tweetResult.result`
//! - 用户：`tweetResult.result.core.user_result.result`
//! - 用户资料：`result.legacy`（新 `core` 字段在时间线里可能为 null）
//! - 引用帖：`result.quoted_status_result.result`
//! - 转推：`result.legacy.retweeted_status_result.result`
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 用户在界面上的所有字段。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XUser {
    pub id: String,
    pub name: String,
    pub handle: String,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub bio: String,
    pub verified: bool,
    pub protected: bool,
    /// 当前登录用户是否已关注此人（来自 legacy.following / relationship_perspectives）
    pub is_following: bool,
    pub followers: u64,
    pub following: u64,
    pub tweets: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XMedia {
    /// photo | video | animated_gif
    pub kind: String,
    pub url: String,
    pub preview: Option<String>,
    /// 视频/动图的最佳可用播放地址
    pub variants: Vec<MediaVariant>,
    pub width: u32,
    pub height: u32,
    pub alt_text: Option<String>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaVariant {
    pub url: String,
    pub content_type: String,
    pub bitrate: Option<u64>,
}

/// 帖子的互动计数。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XMetrics {
    pub reply: u64,
    pub repost: u64,
    pub like: u64,
    pub quote: u64,
    pub bookmark: u64,
    pub view: u64,
}

/// 帖子。引用帖与转推原帖都以嵌套结构保留，便于 UI 递归渲染。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XPost {
    pub id: String,
    pub author: XUser,
    pub text: String,
    /// ISO8601；X 原始格式是 `Fri Nov 13 06:55:52 +0000 2026`，这里已转换。
    pub created_at: String,
    pub lang: String,
    pub metrics: XMetrics,
    pub media: Vec<XMedia>,
    pub is_liked: bool,
    pub is_reposted: bool,
    pub is_bookmarked: bool,
    pub possibly_sensitive: bool,
    /// 引用帖
    pub quoted: Option<Box<XPost>>,
    /// 转推的原帖（`legacy.retweeted_status_result`）
    pub reposted: Option<Box<XPost>>,
    /// 长帖（Note Tweet）的完整正文
    pub note_text: Option<String>,
    pub conversation_id: Option<String>,
    pub in_reply_to_id: Option<String>,
    pub in_reply_to_handle: Option<String>,
}

/// 一页时间线。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TimelinePage {
    pub posts: Vec<XPost>,
    pub top_cursor: Option<String>,
    pub bottom_cursor: Option<String>,
}

/* ---------- 解析辅助 ---------- */

fn str_at<'a>(v: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut cur = v;
    for key in path {
        cur = cur.get(*key)?;
    }
    cur.as_str()
}

fn u64_at(v: &Value, path: &[&str]) -> u64 {
    let mut cur = v;
    for key in path {
        match cur.get(*key) {
            Some(next) => cur = next,
            None => return 0,
        }
    }
    cur.as_u64()
        .or_else(|| cur.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0)
}

fn bool_at(v: &Value, path: &[&str]) -> bool {
    let mut cur = v;
    for key in path {
        match cur.get(*key) {
            Some(next) => cur = next,
            None => return false,
        }
    }
    cur.as_bool().unwrap_or(false)
}

/// 把 X 的 `Fri Nov 13 06:55:52 +0000 2026` 转成 `2026-11-13T06:55:52Z`。
pub fn normalize_created_at(raw: &str) -> String {
    let parts: Vec<&str> = raw.split_whitespace().collect();
    if parts.len() != 6 {
        return raw.to_string();
    }
    let (weekday, month, day, time, tz, year) =
        (parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]);
    if weekday.is_empty() || tz.len() != 5 {
        return raw.to_string();
    }
    let month_num = match month {
        "Jan" => "01",
        "Feb" => "02",
        "Mar" => "03",
        "Apr" => "04",
        "May" => "05",
        "Jun" => "06",
        "Jul" => "07",
        "Aug" => "08",
        "Sep" => "09",
        "Oct" => "10",
        "Nov" => "11",
        "Dec" => "12",
        _ => return raw.to_string(),
    };
    // 时区形如 +0000 / -0700
    let sign = if tz.starts_with('-') { '-' } else { '+' };
    let hours = &tz[1..3];
    let minutes = &tz[3..5];
    format!(
        "{year}-{month_num}-{day}T{time}{sign}{hours}:{minutes}"
    )
}

/// 解析一个 `User` 结果节点（时间线与资料页结构略有差异，这里都兼容）。
pub fn parse_user(node: &Value) -> Option<XUser> {
    let legacy = node.get("legacy").cloned().unwrap_or(Value::Null);
    let core = node.get("core").cloned().unwrap_or(Value::Null);

    let name = str_at(&core, &["name"])
        .or_else(|| str_at(&legacy, &["name"]))
        .unwrap_or("")
        .to_string();
    let handle = str_at(&core, &["screen_name"])
        .or_else(|| str_at(&legacy, &["screen_name"]))
        .unwrap_or("")
        .to_string();

    if name.is_empty() && handle.is_empty() {
        return None;
    }

    // 新结构在 avatar.image_url，旧结构在 legacy.profile_image_url_https
    let avatar = str_at(node, &["avatar", "image_url"])
        .or_else(|| str_at(&legacy, &["profile_image_url_https"]))
        .map(str::to_string);
    let banner = str_at(node, &["banner", "image_url"])
        .or_else(|| str_at(&legacy, &["profile_banner_url"]))
        .map(str::to_string);

    let id = str_at(node, &["rest_id"])
        .or_else(|| str_at(&legacy, &["id_str"]))
        .unwrap_or("")
        .to_string();

    Some(XUser {
        id,
        name,
        handle,
        avatar,
        banner,
        bio: str_at(&legacy, &["description"]).unwrap_or("").to_string(),
        verified: bool_at(node, &["is_blue_verified"]) || bool_at(&legacy, &["verified"]),
        protected: bool_at(&legacy, &["protected"]),
        is_following: bool_at(&legacy, &["following"])
            || bool_at(node, &["relationship_perspectives", "following"]),
        followers: u64_at(&legacy, &["followers_count"]),
        following: u64_at(&legacy, &["friends_count"]),
        tweets: u64_at(&legacy, &["statuses_count"]),
    })
}

fn parse_media(node: &Value) -> Vec<XMedia> {
    let items = node
        .get("extended_entities")
        .or_else(|| node.get("entities"))
        .and_then(|e| e.get("media"))
        .and_then(Value::as_array);

    let Some(items) = items else {
        return Vec::new();
    };

    items
        .iter()
        .map(|m| {
            let kind = m
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("photo")
                .to_string();

            let mut variants = Vec::new();
            let mut duration_ms = None;
            if let Some(info) = m.get("video_info") {
                duration_ms = info.get("duration_millis").and_then(Value::as_u64);
                if let Some(list) = info.get("variants").and_then(Value::as_array) {
                    for v in list {
                        // 只保留可播放的 mp4 变体
                        let ct = v
                            .get("content_type")
                            .and_then(Value::as_str)
                            .unwrap_or("");
                        if !ct.starts_with("video/") {
                            continue;
                        }
                        if let Some(url) = v.get("url").and_then(Value::as_str) {
                            variants.push(MediaVariant {
                                url: url.to_string(),
                                content_type: ct.to_string(),
                                bitrate: v.get("bitrate").and_then(Value::as_u64),
                            });
                        }
                    }
                    // 码率从高到低，前端取第一个即可
                    variants.sort_by_key(|v| std::cmp::Reverse(v.bitrate.unwrap_or(0)));
                }
            }

            let size = m.get("sizes").and_then(|s| s.get("large"));
            XMedia {
                url: str_at(m, &["media_url_https"]).unwrap_or("").to_string(),
                preview: str_at(m, &["media_url_https"]).map(str::to_string),
                width: size.and_then(|s| s.get("w")).and_then(Value::as_u64).unwrap_or(0) as u32,
                height: size.and_then(|s| s.get("h")).and_then(Value::as_u64).unwrap_or(0) as u32,
                alt_text: str_at(m, &["ext_alt_text"]).map(str::to_string),
                kind,
                variants,
                duration_ms,
            }
        })
        .collect()
}

/// 解析一个 `Tweet` / `TweetWithVisibilityResults` 节点。
pub fn parse_post(node: &Value) -> Option<XPost> {
    // 可见性包装层
    let node = if node.get("__typename").and_then(Value::as_str) == Some("TweetWithVisibilityResults") {
        node.get("tweet")?
    } else {
        node
    };

    if node.get("__typename").and_then(Value::as_str) != Some("Tweet") {
        return None;
    }

    let id = str_at(node, &["rest_id"])?.to_string();

    // 作者节点在部分响应里可能缺失（例如已删除账号、或精简返回）。
    // 这时不能让整条帖子消失，用占位作者兜底。
    // 两种命名都出现过：时间线是 user_result，详情页是 user_results
    let author = node
        .get("core")
        .and_then(|c| c.get("user_result").or_else(|| c.get("user_results")))
        .and_then(|u| u.get("result"))
        .and_then(parse_user)
        .unwrap_or_else(|| XUser {
            id: String::new(),
            name: "未知用户".to_string(),
            handle: String::new(),
            ..Default::default()
        });

    let legacy = node.get("legacy").cloned().unwrap_or(Value::Null);

    // 长帖优先取 note_tweet 的完整正文
    let note_text = node
        .get("note_tweet")
        .and_then(|n| n.get("note_tweet_results"))
        .and_then(|n| n.get("result"))
        .and_then(|r| r.get("text"))
        .and_then(Value::as_str)
        .map(str::to_string);

    let text = note_text
        .clone()
        .unwrap_or_else(|| str_at(&legacy, &["full_text"]).unwrap_or("").to_string());

    // 转推：原帖在 legacy.retweeted_status_result.result
    let reposted = legacy
        .get("retweeted_status_result")
        .and_then(|r| r.get("result"))
        .and_then(parse_post)
        .map(Box::new);

    // 引用：quoted_status_result.result 或 quoted_tweet_results.result
    let quoted = node
        .get("quoted_status_result")
        .and_then(|r| r.get("result"))
        .or_else(|| node.get("quoted_tweet_results").and_then(|r| r.get("result")))
        .and_then(parse_post)
        .map(Box::new);

    let in_reply_to_id = legacy
        .get("in_reply_to_status_id_str")
        .and_then(Value::as_str)
        .map(str::to_string);
    let in_reply_to_handle = legacy
        .get("in_reply_to_screen_name")
        .and_then(Value::as_str)
        .map(str::to_string);

    Some(XPost {
        id,
        author,
        text,
        created_at: normalize_created_at(str_at(&legacy, &["created_at"]).unwrap_or("")),
        lang: str_at(&legacy, &["lang"]).unwrap_or("").to_string(),
        metrics: XMetrics {
            reply: u64_at(&legacy, &["reply_count"]),
            repost: u64_at(&legacy, &["retweet_count"]),
            like: u64_at(&legacy, &["favorite_count"]),
            quote: u64_at(&legacy, &["quote_count"]),
            bookmark: u64_at(&legacy, &["bookmark_count"]),
            view: u64_at(node, &["views", "count"]),
        },
        media: parse_media(&legacy),
        is_liked: bool_at(&legacy, &["favorited"]),
        is_reposted: bool_at(&legacy, &["retweeted"]),
        is_bookmarked: bool_at(&legacy, &["bookmarked"]),
        possibly_sensitive: bool_at(&legacy, &["possibly_sensitive"]),
        quoted,
        reposted,
        note_text,
        conversation_id: legacy
            .get("conversation_id_str")
            .and_then(Value::as_str)
            .map(str::to_string),
        in_reply_to_id,
        in_reply_to_handle,
    })
}

/// 解析 `HomeTimeline` / `HomeTimelineLatest` / `SearchTimeline` 的响应。
///
/// 时间线结构：`data.{timeline_response|search_by_raw_query...}.timeline.instructions[].entries[]`
/// 每个 entry 可能是 `TimelineTimelineItem`（帖子）或 `TimelineTimelineCursor`（分页游标）。
pub fn parse_timeline(data: &Value) -> TimelinePage {
    let mut page = TimelinePage::default();

    // 找到第一个带 instructions 的 timeline 节点
    let mut timeline = None;
    if let Some(root) = data.get("data") {
        timeline = find_timeline(root);
    }
    let Some(timeline) = timeline else {
        return page;
    };

    let Some(instructions) = timeline.get("instructions").and_then(Value::as_array) else {
        return page;
    };

    for instruction in instructions {
        let Some(entries) = instruction.get("entries").and_then(Value::as_array) else {
            continue;
        };
        for entry in entries {
            let entry_id = entry.get("entryId").and_then(Value::as_str).unwrap_or("");
            let content = entry.get("content").cloned().unwrap_or(Value::Null);
            let typename = content.get("__typename").and_then(Value::as_str).unwrap_or("");

            if typename == "TimelineTimelineCursor" {
                let cursor_type = content.get("cursorType").and_then(Value::as_str).unwrap_or("");
                let value = content
                    .get("value")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                match cursor_type {
                    "Top" => page.top_cursor = value,
                    "Bottom" => page.bottom_cursor = value,
                    _ => {
                        // 兜底：用 entryId 前缀判断
                        if entry_id.starts_with("cursor-top") {
                            page.top_cursor = content
                                .get("value")
                                .and_then(Value::as_str)
                                .map(str::to_string);
                        } else if entry_id.starts_with("cursor-bottom") {
                            page.bottom_cursor = content
                                .get("value")
                                .and_then(Value::as_str)
                                .map(str::to_string);
                        }
                    }
                }
                continue;
            }

            // TimelineTimelineItem -> content.content.tweetResult.result
            let tweet = content
                .get("content")
                .and_then(|c| c.get("tweetResult"))
                .and_then(|t| t.get("result"));
            if let Some(node) = tweet {
                if let Some(post) = parse_post(node) {
                    page.posts.push(post);
                }
            }
        }
    }

    page
}

/// 递归查找 `timeline` 节点（不同查询外层包装不同）。
fn find_timeline(node: &Value) -> Option<&Value> {
    if let Some(tl) = node.get("timeline") {
        if tl.get("instructions").is_some() {
            return Some(tl);
        }
    }
    match node {
        Value::Object(map) => {
            for value in map.values() {
                if let Some(found) = find_timeline(value) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(list) => {
            for value in list {
                if let Some(found) = find_timeline(value) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

/// 解析 `UserByScreenName` 响应。
pub fn parse_user_by_screen_name(data: &Value) -> Option<XUser> {
    let node = data
        .get("data")?
        .get("user")?
        .get("result")?;
    parse_user(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn normalizes_x_date() {
        assert_eq!(
            normalize_created_at("Fri Nov 13 06:55:52 +0000 2026"),
            "2026-11-13T06:55:52+00:00"
        );
    }

    #[test]
    fn parses_minimal_tweet() {
        let node = json!({
            "__typename": "Tweet",
            "rest_id": "123",
            "core": { "user_result": { "result": {
                "__typename": "User",
                "rest_id": "u1",
                "is_blue_verified": true,
                "legacy": { "name": "测试", "screen_name": "test", "profile_image_url_https": "https://a/b.jpg" }
            }}},
            "legacy": {
                "full_text": "hello",
                "created_at": "Fri Nov 13 06:55:52 +0000 2026",
                "lang": "en",
                "favorite_count": 3,
                "bookmark_count": 1
            }
        });
        let post = parse_post(&node).expect("应能解析");
        assert_eq!(post.id, "123");
        assert_eq!(post.text, "hello");
        assert_eq!(post.author.handle, "test");
        assert!(post.author.verified);
        assert_eq!(post.metrics.like, 3);
        assert_eq!(post.created_at, "2026-11-13T06:55:52+00:00");
    }

    #[test]
    fn unwraps_visibility_wrapper() {
        let node = json!({
            "__typename": "TweetWithVisibilityResults",
            "tweet": {
                "__typename": "Tweet",
                "rest_id": "9",
                "core": { "user_result": { "result": {
                    "rest_id": "u", "legacy": { "name": "n", "screen_name": "h" }
                }}},
                "legacy": { "full_text": "wrapped" }
            }
        });
        let post = parse_post(&node).expect("应能解包");
        assert_eq!(post.text, "wrapped");
    }

    #[test]
    fn parses_cursor_and_posts_from_timeline() {
        let data = json!({
            "data": { "timeline_response": { "timeline": {
                "instructions": [{ "entries": [
                    { "entryId": "cursor-top-1", "content": {
                        "__typename": "TimelineTimelineCursor", "cursorType": "Top", "value": "TOP"
                    }},
                    { "entryId": "tweet-1", "content": {
                        "__typename": "TimelineTimelineItem",
                        "content": { "__typename": "TimelineTweet", "tweetResult": { "result": {
                            "__typename": "Tweet", "rest_id": "1",
                            "core": { "user_result": { "result": {
                                "rest_id": "u", "legacy": { "name": "n", "screen_name": "h" }
                            }}},
                            "legacy": { "full_text": "hi" }
                        }}}
                    }},
                    { "entryId": "cursor-bottom-1", "content": {
                        "__typename": "TimelineTimelineCursor", "cursorType": "Bottom", "value": "BOT"
                    }}
                ]}]
            }}}
        });
        let page = parse_timeline(&data);
        assert_eq!(page.posts.len(), 1);
        assert_eq!(page.top_cursor.as_deref(), Some("TOP"));
        assert_eq!(page.bottom_cursor.as_deref(), Some("BOT"));
    }
}
/* ---------- 通知 ---------- */

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XNotification {
    pub id: String,
    /// like | retweet | follow | mention | reply | quote | other
    pub kind: String,
    pub actors: Vec<XUser>,
    /// 关联的帖子（提及/回复/点赞等）
    pub post: Option<XPost>,
    /// 通知本身的文案（X 提供了 message）
    pub message: Option<String>,
    pub created_at: Option<String>,
}

/// 解析通知时间线。
///
/// 实测结构（2026-11-15）：
/// `entry.content.content` → `TimelineNotification`，含：
/// - `rich_message.text`：X 已渲染好的文案（例如「xx 已接受你的关注请求」）
/// - `template.__typename`：通知类型
/// - `target_tweets_results`：关联帖子
/// - `timestamp_ms`：时间戳
/// - `icon`：图标标识
///
/// 注意这一层用的是下划线命名，与时间线接口不同。
pub fn parse_notifications(data: &Value) -> Vec<XNotification> {
    let mut out = Vec::new();
    let Some(timeline) = data.get("data").and_then(find_timeline) else {
        return out;
    };
    let Some(instructions) = timeline.get("instructions").and_then(Value::as_array) else {
        return out;
    };

    for instruction in instructions {
        let Some(entries) = instruction.get("entries").and_then(Value::as_array) else {
            continue;
        };
        for entry in entries {
            let outer = entry.get("content").cloned().unwrap_or(Value::Null);
            // 通知在 content.content；游标等其它 entry 直接跳过
            let inner = match outer.get("content") {
                Some(v) if v.get("__typename").and_then(Value::as_str) == Some("TimelineNotification") => v,
                _ => continue,
            };

            let kind = classify_notification(inner);
            let message = inner
                .pointer("/rich_message/text")
                .and_then(Value::as_str)
                .map(str::to_string);

            let created_at = inner
                .get("timestamp_ms")
                .and_then(Value::as_str)
                .and_then(|ms| ms.parse::<i64>().ok())
                .map(|ms| chrono_like_from_unix(ms / 1000));

            // 关联帖子：target_tweets_results 是数组
            let post = inner
                .get("target_tweets_results")
                .and_then(Value::as_array)
                .and_then(|list| list.first())
                .and_then(|t| t.get("result"))
                .and_then(parse_post);

            let id = inner
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();

            if id.is_empty() && message.is_none() {
                continue;
            }

            out.push(XNotification {
                id,
                kind,
                actors: Vec::new(),
                post,
                message,
                created_at,
            });
        }
    }
    out
}

/// 从 `template.__typename` 与 `icon` 推断通知类型。
fn classify_notification(node: &Value) -> String {
    let template = node
        .pointer("/template/__typename")
        .and_then(Value::as_str)
        .unwrap_or("");
    let icon = node.get("icon").and_then(Value::as_str).unwrap_or("");
    let haystack = format!("{template} {icon}").to_lowercase();

    // 图标标识（X 用的是 xxx_icon 形式）与 template 类型都要覆盖
    for (needle, kind) in [
        ("heart", "like"),
        ("favorite", "like"),
        ("like", "like"),
        ("retweet", "repost"),
        ("repost", "repost"),
        ("person", "follow"),
        ("follow", "follow"),
        ("mention", "mention"),
        ("reply", "reply"),
        ("quote", "quote"),
        ("list", "list"),
        ("community", "community"),
    ] {
        if haystack.contains(needle) {
            return kind.to_string();
        }
    }
    "other".to_string()
}

/// 毫秒时间戳 → ISO8601。
pub fn unix_ms_to_iso(ms: i64) -> String {
    chrono_like_from_unix(ms / 1000)
}

/// 最小化的 Unix 秒 → ISO8601（避免引入 chrono 依赖）。
fn chrono_like_from_unix(secs: i64) -> String {
    // 以 1970 起算的民用历算法（Howard Hinnant's days_from_civil 的逆运算）
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    format!("{year:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// 解析 `TweetDetail` 响应。
///
/// 真实结构（2026-11-15 实测）：
/// `data.threaded_conversation_with_injections_v2.instructions[].entries[]`
/// - 主帖与直接回复：`entry.content.itemContent.tweet_results.result`
/// - 折叠的回复串：`entry.content.items[].item.itemContent.tweet_results.result`
///
/// 注意这一层用下划线命名（`tweet_results`），与首页时间线的
/// `tweetResult` 不同。
pub fn parse_tweet_detail(data: &Value) -> (Option<XPost>, Vec<XPost>) {
    let mut main: Option<XPost> = None;
    let mut replies: Vec<XPost> = Vec::new();

    let Some(instructions) = data
        .get("data")
        .and_then(|d| d.get("threaded_conversation_with_injections_v2"))
        .and_then(|t| t.get("instructions"))
        .and_then(Value::as_array)
    else {
        return (None, replies);
    };

    for instruction in instructions {
        let Some(entries) = instruction.get("entries").and_then(Value::as_array) else {
            continue;
        };
        for entry in entries {
            let content = entry.get("content").cloned().unwrap_or(Value::Null);

            // 直接条目
            if let Some(item) = content.get("itemContent") {
                if let Some(post) = post_from_item(item) {
                    if main.is_none() {
                        main = Some(post);
                    } else {
                        replies.push(post);
                    }
                }
            }

            // 折叠的回复串
            if let Some(items) = content.get("items").and_then(Value::as_array) {
                for sub in items {
                    if let Some(item) = sub.pointer("/item/itemContent") {
                        if let Some(post) = post_from_item(item) {
                            replies.push(post);
                        }
                    }
                }
            }
        }
    }

    (main, replies)
}

/// 从 `itemContent` 里取出帖子（兼容 `tweet_results` 与 `tweetResult`）。
fn post_from_item(item: &Value) -> Option<XPost> {
    item.get("tweet_results")
        .or_else(|| item.get("tweetResult"))
        .and_then(|t| t.get("result"))
        .and_then(parse_post)
}

#[cfg(test)]
mod extra_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn converts_unix_to_iso() {
        assert_eq!(chrono_like_from_unix(0), "1970-01-01T00:00:00Z");
        // 2026-11-13T06:55:52Z
        assert_eq!(chrono_like_from_unix(1_763_016_952), "2025-11-13T06:55:52Z");
    }

    #[test]
    fn parses_notification_entry() {
        // 结构来自 2026-11-15 的真实通知响应（下划线命名）
        let data = json!({
            "data": { "user_result": { "result": { "timeline_response": { "timeline": {
                "instructions": [{ "entries": [
                    { "entryId": "cursor-top", "content": {
                        "__typename": "TimelineTimelineCursor", "cursor_type": "Top", "value": "X"
                    }},
                    { "entryId": "notif-1", "content": {
                        "__typename": "TimelineTimelineItem",
                        "content": {
                            "__typename": "TimelineNotification",
                            "id": "N1",
                            "icon": "heart_icon",
                            "rich_message": { "__typename": "TimelineRichText", "text": "某人 赞了你的帖子" },
                            "template": { "__typename": "TimelineNotificationAggregateUserActions" },
                            "timestamp_ms": "1763016952000",
                            "target_tweets_results": [{ "result": {
                                "__typename": "Tweet", "rest_id": "t1",
                                "core": { "user_result": { "result": {
                                    "rest_id": "u2", "legacy": { "name": "作者", "screen_name": "author" }
                                }}},
                                "legacy": { "full_text": "被赞的帖子" }
                            }}]
                        }
                    }}
                ]}]
            }}}}}
        });
        let list = parse_notifications(&data);
        assert_eq!(list.len(), 1, "应解析出 1 条通知");
        assert_eq!(list[0].kind, "like");
        assert_eq!(list[0].message.as_deref(), Some("某人 赞了你的帖子"));
        assert_eq!(list[0].post.as_ref().unwrap().text, "被赞的帖子");
        assert_eq!(list[0].created_at.as_deref(), Some("2025-11-13T06:55:52Z"));
    }
}

#[cfg(test)]
mod detail_real_test {
    use super::*;

    /// 用真实 TweetDetail 响应验证回复作者能正确解析。
    #[test]
    fn parses_real_detail_authors() {
        let Ok(raw) = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/detail_sample.json")) else {
            eprintln!("跳过：缺少 detail_sample.json");
            return;
        };
        let data: Value = serde_json::from_str(&raw).expect("样本 JSON 解析失败");
        let (main, replies) = parse_tweet_detail(&data);
        assert!(main.is_some(), "主帖应能解析");
        assert!(!replies.is_empty(), "应有回复");
        let unknown = replies
            .iter()
            .filter(|p| p.author.name == "未知用户")
            .count();
        assert_eq!(unknown, 0, "不应有未知用户，实际 {} 条", unknown);
        println!("主帖 @{}，回复 {} 条，作者全部解析", main.unwrap().author.handle, replies.len());
    }
}
