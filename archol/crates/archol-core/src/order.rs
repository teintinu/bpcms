//! Canonical AST property order.
//!
//! JSON, YAML, the LSP and the AST explorer all read the same
//! [`ast-order.json`](../../../ast-order.json): the first keys keep a
//! semantic order, `loc`-shaped keys go last, and every other key is
//! alphabetical.

use std::sync::OnceLock;

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrderConfig {
    first_keys: Vec<String>,
    loc_suffix: String,
    /// Documentation only: the current implementation always breaks ties by key.
    #[allow(dead_code)]
    fallback: String,
}

const ORDER_JSON: &str = include_str!("../../../ast-order.json");

fn config() -> &'static OrderConfig {
    static CONFIG: OnceLock<OrderConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
        serde_json::from_str(ORDER_JSON).expect("archol/ast-order.json must be valid")
    })
}

/// Rank of a key: first keys, then normal keys, then location keys.
pub fn key_rank(key: &str) -> usize {
    let config = config();
    if let Some(index) = config
        .first_keys
        .iter()
        .position(|candidate| candidate == key)
    {
        return index;
    }
    if key
        .to_ascii_lowercase()
        .ends_with(&config.loc_suffix.to_ascii_lowercase())
    {
        return 1000;
    }
    100
}

/// Rebuilds every object in the canonical order, recursively.
pub fn order_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = std::mem::take(map).into_iter().collect();
            for (_, item) in &mut entries {
                order_value(item);
            }
            entries.sort_by(|(left, _), (right, _)| {
                key_rank(left)
                    .cmp(&key_rank(right))
                    .then_with(|| left.cmp(right))
            });
            for (key, item) in entries {
                map.insert(key, item);
            }
        }
        Value::Array(items) => {
            for item in items {
                order_value(item);
            }
        }
        _ => {}
    }
}

/// Convenience for callers that own a value.
pub fn ordered(value: Value) -> Value {
    let mut value = value;
    order_value(&mut value);
    value
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn first_keys_then_alpha_then_loc() {
        let mut value = json!({
            "loc": { "start": 0 },
            "z": 1,
            "kind": "X",
            "name": "N",
            "a": 2,
            "keywordLoc": { "start": 0 }
        });
        order_value(&mut value);
        let keys: Vec<_> = value.as_object().unwrap().keys().cloned().collect();
        assert_eq!(keys, ["name", "kind", "a", "z", "keywordLoc", "loc"]);
    }

    #[test]
    fn recurses_into_arrays_and_objects() {
        let mut value = json!([{ "z": 1, "kind": "A" }]);
        order_value(&mut value);
        let keys: Vec<_> = value[0].as_object().unwrap().keys().cloned().collect();
        assert_eq!(keys, ["kind", "z"]);
    }
}
