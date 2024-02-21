mod json_selector;
mod messagepack;
mod tree_items_from_json;
mod tree_items_from_messagepack;

pub use json_selector::JsonSelector;
pub use tree_items_from_json::tree_items_from_json;
pub use tree_items_from_messagepack::tree_items_from_messagepack;

#[derive(Debug, Clone, PartialEq)]
pub enum Payload {
    Json(serde_json::Value),
    MessagePack(rmpv::Value),
    NotUtf8(std::str::Utf8Error),
    String(Box<str>),
}

impl Payload {
    pub fn new(payload: Vec<u8>) -> Self {
        match String::from_utf8(payload) {
            Ok(str) => {
                serde_json::from_str(&str).map_or_else(|_| Self::String(str.into()), Self::Json)
            }
            Err(err) => messagepack::decode(err.as_bytes())
                .map_or_else(|| Self::NotUtf8(err.utf8_error()), Self::MessagePack),
        }
    }
}

impl std::fmt::Display for Payload {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(json) => json.fmt(fmt),
            Self::MessagePack(messagepack) => messagepack.fmt(fmt),
            Self::NotUtf8(err) => write!(fmt, "not valid UTF-8: {err}"),
            Self::String(str) => str.fmt(fmt),
        }
    }
}

#[test]
fn display_json_works() {
    let payload = Payload::Json(serde_json::json!([42, false]));
    assert_eq!(format!("{payload}"), "[42,false]");
}

#[test]
fn display_string_works() {
    let payload = Payload::String("bar".into());
    assert_eq!(format!("{payload}"), "bar");
}

#[cfg(test)]
fn json_macro(json_str: &'static str) -> Option<String> {
    match Payload::new(json_str.into()) {
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
