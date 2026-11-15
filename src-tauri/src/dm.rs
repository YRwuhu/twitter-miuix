//! 私信（旧版 REST DM 接口）。
//!
//! 实测（2026-11-15）：`GET /1.1/dm/inbox_initial_state.json` 返回 200，
//! 含真实会话与消息。XChat 那套 GraphQL 接口需要端到端加密状态机，
//! 这里只做基础私信能力。
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::XUser;

/// 一个私信会话。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DmConversation {
    pub id: String,
    /// ONE_TO_ONE | GROUP
    pub kind: String,
    /// 会话参与者（不含自己）
    pub participants: Vec<XUser>,
    pub last_message: Option<String>,
    /// ISO8601
    pub last_time: Option<String>,
    pub unread: bool,
    pub muted: bool,
}

/// 一条私信消息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DmMessage {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub text: String,
    /// ISO8601
    pub time: String,
    pub from_me: bool,
}

/// 解析 `inbox_initial_state`。
pub fn parse_inbox(root: &Value, my_id: &str) -> Vec<DmConversation> {
    let state = match root.get("inbox_initial_state") {
        Some(s) => s,
        None => return Vec::new(),
    };

    let mut users: std::collections::HashMap<String, XUser> = std::collections::HashMap::new();
    if let Some(map) = state.get("users").and_then(Value::as_object) {
        for u in map.values() {
            if let Some(id) = u.get("id_str").and_then(Value::as_str) {
                users.insert(id.to_string(), parse_dm_user(u));
            }
        }
    }

    // conversation_id -> (最后文本, 最后时间)
    let mut last: std::collections::HashMap<String, (String, String)> =
        std::collections::HashMap::new();
    if let Some(entries) = state.get("entries").and_then(Value::as_array) {
        for entry in entries {
            let Some(msg) = entry.get("message") else { continue };
            let Some(cid) = msg.get("conversation_id").and_then(Value::as_str) else {
                continue;
            };
            let text = msg
                .pointer("/message_data/text")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let time = msg
                .pointer("/message_data/time")
                .and_then(Value::as_str)
                .and_then(|t| t.parse::<i64>().ok())
                .map(crate::models::unix_ms_to_iso)
                .unwrap_or_default();
            let replace = last
                .get(cid)
                .map(|(_, prev)| time > *prev)
                .unwrap_or(true);
            if replace {
                last.insert(cid.to_string(), (text, time));
            }
        }
    }

    let mut out = Vec::new();
    if let Some(map) = state.get("conversations").and_then(Value::as_object) {
        for (cid, conv) in map {
            let kind = conv
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("ONE_TO_ONE")
                .to_string();

            let mut participants = Vec::new();
            if let Some(list) = conv.get("participants").and_then(Value::as_array) {
                for p in list {
                    let Some(uid) = p.get("user_id").and_then(Value::as_str) else {
                        continue;
                    };
                    if uid == my_id {
                        continue;
                    }
                    if let Some(u) = users.get(uid) {
                        participants.push(u.clone());
                    }
                }
            }

            let (last_message, last_time) = last.get(cid).cloned().unwrap_or_default();
            out.push(DmConversation {
                id: cid.clone(),
                kind,
                participants,
                last_message: (!last_message.is_empty()).then_some(last_message),
                last_time: (!last_time.is_empty()).then_some(last_time),
                // 未读判断依赖 read 游标，这里保守地不标记，避免误报
                unread: false,
                muted: conv.get("muted").and_then(Value::as_bool).unwrap_or(false),
            });
        }
    }

    out.sort_by(|a, b| b.last_time.cmp(&a.last_time));
    out
}

/// 解析会话消息（`/1.1/dm/conversation/{id}.json`）。
pub fn parse_conversation(root: &Value, my_id: &str) -> Vec<DmMessage> {
    let mut out = Vec::new();
    // 实测结构是 `conversation_timeline.entries`，
    // 兼容旧文档里的 `conversation.events`。
    let entries = root
        .pointer("/conversation_timeline/entries")
        .or_else(|| root.pointer("/conversation/events"))
        .and_then(Value::as_array);
    let Some(entries) = entries else {
        return out;
    };

    for ev in entries {
        let Some(msg) = ev.get("message") else { continue };
        let Some(data) = msg.get("message_data") else {
            continue;
        };
        let sender = data
            .get("sender_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        out.push(DmMessage {
            id: msg
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            conversation_id: msg
                .get("conversation_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            from_me: sender == my_id,
            sender_id: sender,
            text: data
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            time: data
                .get("time")
                .and_then(Value::as_str)
                .and_then(|t| t.parse::<i64>().ok())
                .map(crate::models::unix_ms_to_iso)
                .unwrap_or_default(),
        });
    }

    out.sort_by(|a, b| a.time.cmp(&b.time));
    out
}

/// REST user 对象 → XUser（字段名与 GraphQL 不同）。
fn parse_dm_user(u: &Value) -> XUser {
    let s = |k: &str| {
        u.get(k)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    XUser {
        id: s("id_str"),
        name: s("name"),
        handle: s("screen_name"),
        avatar: Some(s("profile_image_url_https")).filter(|v| !v.is_empty()),
        banner: None,
        bio: s("description"),
        verified: u.get("verified").and_then(Value::as_bool).unwrap_or(false),
        protected: u.get("protected").and_then(Value::as_bool).unwrap_or(false),
        is_following: false,
        followers: u.get("followers_count").and_then(Value::as_u64).unwrap_or(0),
        following: u.get("friends_count").and_then(Value::as_u64).unwrap_or(0),
        tweets: u.get("statuses_count").and_then(Value::as_u64).unwrap_or(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_inbox_conversations() {
        let root = json!({
            "inbox_initial_state": {
                "users": {
                    "me": { "id_str": "me", "name": "我", "screen_name": "me" },
                    "other": { "id_str": "other", "name": "对方", "screen_name": "other",
                               "profile_image_url_https": "https://a/b.jpg" }
                },
                "entries": [{
                    "message": {
                        "conversation_id": "me-other",
                        "id": "m1",
                        "message_data": { "text": "你好", "time": "1763016952000", "sender_id": "other" }
                    }
                }],
                "conversations": {
                    "me-other": {
                        "conversation_id": "me-other",
                        "type": "ONE_TO_ONE",
                        "participants": [{ "user_id": "me" }, { "user_id": "other" }],
                        "muted": false
                    }
                }
            }
        });
        let list = parse_inbox(&root, "me");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].participants.len(), 1, "不应包含自己");
        assert_eq!(list[0].participants[0].handle, "other");
        assert_eq!(list[0].last_message.as_deref(), Some("你好"));
    }

    #[test]
    fn parses_conversation_messages() {
        let root = json!({
            "conversation_timeline": {
                "entries": [
                    { "message": { "id": "m1", "conversation_id": "c",
                        "message_data": { "sender_id": "other", "text": "hi", "time": "1000" } } },
                    { "message": { "id": "m2", "conversation_id": "c",
                        "message_data": { "sender_id": "me", "text": "hello", "time": "2000" } } }
                ]
            }
        });
        let list = parse_conversation(&root, "me");
        assert_eq!(list.len(), 2);
        assert!(!list[0].from_me);
        assert!(list[1].from_me);
        assert_eq!(list[1].text, "hello");
    }
}
