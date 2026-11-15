//! queryId 注册表。
//!
//! 全部 309 条来自静态提取（`data/gql_ops.json`）。
//! X 会随版本轮换这些 ID；失效时返回 404，可在设置里覆盖并持久化。
use std::collections::HashMap;
use std::sync::OnceLock;

use serde_json::Value;

const RAW: &str = include_str!("../data/gql_ops.json");

fn table() -> &'static HashMap<String, String> {
    static TABLE: OnceLock<HashMap<String, String>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut map = HashMap::new();
        if let Ok(Value::Object(ops)) = serde_json::from_str::<Value>(RAW) {
            for (name, entry) in ops {
                if let Some(id) = entry.get("queryId").and_then(Value::as_str) {
                    map.insert(name, id.to_string());
                }
            }
        }
        map
    })
}

/// 取某个操作的 queryId。`overrides` 优先于内置表。
pub fn get(operation: &str, overrides: &HashMap<String, String>) -> Option<String> {
    overrides
        .get(operation)
        .cloned()
        .or_else(|| table().get(operation).cloned())
}

/// 内置表里的全部操作（用于「接口」面板展示）。
pub fn all() -> HashMap<String, String> {
    table().clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_expected_operations() {
        let all = all();
        assert!(all.len() > 250, "应加载 300+ 操作，实际 {}", all.len());
        assert_eq!(
            all.get("HomeTimeline").map(String::as_str),
            Some("ob5uGlwVI-iAKKFx9YXoqg")
        );
        assert_eq!(
            all.get("UserByScreenName").map(String::as_str),
            Some("IGgvgiOx4QZndDHuD3x9TQ")
        );
    }

    #[test]
    fn overrides_take_precedence() {
        let mut overrides = HashMap::new();
        overrides.insert("HomeTimeline".to_string(), "NEW_ID".to_string());
        assert_eq!(get("HomeTimeline", &overrides).as_deref(), Some("NEW_ID"));
        assert_eq!(
            get("UserByScreenName", &overrides).as_deref(),
            Some("IGgvgiOx4QZndDHuD3x9TQ")
        );
    }
}