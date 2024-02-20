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
            Err(err) => decode_messagepack(err.as_bytes())
                .map_or_else(|| Self::NotUtf8(err.utf8_error()), Self::MessagePack),
        }
    }

    pub fn format_oneline(&self, size: usize) -> String {
        match self {
            Self::Json(json) => format!("Payload({size:>3}): {json}"),
            Self::MessagePack(messagepack) => format!("Payload({size:>3}): {messagepack}"),
            Self::NotUtf8(err) => format!("Payload({size:>3}) is not valid UTF-8: {err}"),
            Self::String(str) => format!("Payload({size:>3}): {str}"),
        }
    }
}

/// Attempts to decode [`MessagePack`](rmpv::Value) from the payload.
/// Tries to find out if data seems valid.
pub fn decode_messagepack(mut payload: &[u8]) -> Option<rmpv::Value> {
    fn has_duplicate_keys(value: &rmpv::Value) -> bool {
        match value {
            rmpv::Value::Nil
            | rmpv::Value::Boolean(_)
            | rmpv::Value::Integer(_)
            | rmpv::Value::F32(_)
            | rmpv::Value::F64(_)
            | rmpv::Value::String(_)
            | rmpv::Value::Binary(_)
            | rmpv::Value::Ext(_, _) => false,
            rmpv::Value::Array(values) => values.iter().any(has_duplicate_keys),
            rmpv::Value::Map(map) => {
                let mut keys = map
                    .iter()
                    .map(|(key, _value)| {
                        key.as_str()
                            .map_or_else(|| key.to_string(), ToOwned::to_owned)
                    })
                    .collect::<Vec<_>>();
                let before = keys.len();
                keys.sort_unstable();
                keys.dedup();
                if before > keys.len() {
                    return true;
                }
                map.iter()
                    .map(|(_key, value)| value)
                    .any(has_duplicate_keys)
            }
        }
    }

    let value = rmpv::decode::read_value(&mut payload).ok()?;
    if value.to_string().len() < payload.len() {
        // The JSON should be bigger than the bytes.
        // Otherwise there is data missing from the bytes so its unlikely MessagePack.
        return None;
    }
    if has_duplicate_keys(&value) {
        // Can not be displayed by the Tree widget currently anyway
        return None;
    }
    Some(value)
}

#[test]
fn oneline_json_works() {
    let payload = Payload::Json(serde_json::json!([42, false]));
    assert_eq!(payload.format_oneline(666), "Payload(666): [42,false]");
}

#[test]
fn oneline_string_works() {
    let payload = Payload::String("bar".into());
    assert_eq!(payload.format_oneline(3), "Payload(  3): bar");
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
