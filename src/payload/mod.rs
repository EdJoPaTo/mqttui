use serde::Serialize;

pub use self::json::tree_items as tree_items_from_json;
pub use self::json_selector::JsonSelector;
pub use self::messagepack::tree_items::tree_items as tree_items_from_messagepack;

mod json;
mod json_selector;
mod messagepack;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Payload {
    /// Might be truncated
    Binary(Box<[u8]>),
    Json(serde_json::Value),
    MessagePack(rmpv::Value),
    /// Might be truncated
    String(Box<str>),
}

impl Payload {
    pub fn truncated(mut payload: Vec<u8>, limit: usize) -> Self {
        if payload.len() > limit {
            payload.truncate(limit);

            match String::from_utf8(payload) {
                Ok(str) => Self::String(str.into()),
                Err(err) => Self::Binary(err.into_bytes().into()),
            }
        } else {
            Self::unlimited(payload)
        }
    }

    pub fn unlimited(payload: Vec<u8>) -> Self {
        match String::from_utf8(payload) {
            Ok(str) => {
                serde_json::from_str(&str).map_or_else(|_| Self::String(str.into()), Self::Json)
            }
            Err(err) => messagepack::decode(err.as_bytes())
                .map_or_else(|| Self::Binary(err.into_bytes().into()), Self::MessagePack),
        }
    }
}

impl std::fmt::Display for Payload {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binary(binary) => std::fmt::Debug::fmt(&binary, fmt),
            Self::Json(json) => json.fmt(fmt),
            Self::MessagePack(messagepack) => messagepack.fmt(fmt),
            Self::String(str) => str.fmt(fmt),
        }
    }
}

#[test]
fn truncates_string() {
    let payload = b"hello world".into();
    let payload = Payload::truncated(payload, 5);
    assert_eq!(payload, Payload::String("hello".into()));
}

#[test]
fn doesnt_truncate_short_string() {
    let payload = b"hello world".into();
    let payload = Payload::truncated(payload, 20);
    assert_eq!(payload, Payload::String("hello world".into()));
}

#[test]
fn truncates_binary() {
    let payload = vec![0, 159, 146, 150, 42];
    let payload = Payload::truncated(payload, 4);
    assert_eq!(payload, Payload::Binary([0, 159, 146, 150].into()));
}

#[test]
fn unlimited_binary() {
    let payload = vec![0, 159, 146, 150];
    let payload = Payload::unlimited(payload);
    assert_eq!(payload, Payload::Binary([0, 159, 146, 150].into()));
}

#[test]
fn display_binary_works() {
    let payload = Payload::Binary([1, 3, 3, 7].into());
    assert_eq!(format!("{payload}"), "[1, 3, 3, 7]");
}

#[test]
fn display_json_works() {
    let payload = Payload::Json(serde_json::json!([42, false]));
    assert_eq!(format!("{payload}"), "[42,false]");
}

#[test]
fn display_messagepack_works() {
    use rmpv::Value;
    let payload = Payload::MessagePack(Value::Array(vec![
        Value::Integer(42.into()),
        Value::Boolean(false),
    ]));
    assert_eq!(format!("{payload}"), "[42, false]");
}

#[test]
fn display_string_works() {
    let payload = Payload::String("bar".into());
    assert_eq!(format!("{payload}"), "bar");
}

#[cfg(test)]
fn json_macro(json_str: &'static str) -> Option<String> {
    match Payload::unlimited(json_str.into()) {
        Payload::Json(json) => Some(json.to_string()),
        _ => None,
    }
}

#[test]
fn pretty_json_ignores_plain() {
    assert_eq!(None, json_macro("bob"));
}

#[test]
fn pretty_json_object_works() {
    assert_eq!(
        json_macro(r#"{"a": "alpha", "b": "beta"}"#),
        Some(r#"{"a":"alpha","b":"beta"}"#.to_owned())
    );
}

#[test]
fn pretty_json_number_works() {
    assert_eq!(json_macro("42"), Some("42".to_owned()));
}
