#[derive(Debug, Clone, PartialEq)]
pub enum Payload {
    Json(serde_json::Value),
    MsgPack(rmpv::Value, serde_json::Value),
    NotUtf8(std::str::Utf8Error),
    String(Box<str>),
}

impl Payload {
    pub fn new(payload: Vec<u8>) -> Self {
        match String::from_utf8(payload) {
            Ok(str) => {
                serde_json::from_str(&str).map_or_else(|_| Self::String(str.into()), Self::Json)
            }
            Err(err) => {
                let mut payload = err.as_bytes();
                match rmpv::decode::read_value(&mut payload) {
                    Ok(value) => {
                        let string = value.to_string();
                        serde_json::from_str(&string).map_or_else(
                            |_| Self::String(string.into()),
                            |json| Self::MsgPack(value, json),
                        )
                    }
                    Err(_) => Self::NotUtf8(err.utf8_error()),
                }
            }
        }
    }

    pub const fn as_optional_json(&self) -> Option<&serde_json::Value> {
        if let Self::Json(json) = self {
            Some(json)
        } else {
            None
        }
    }

    pub fn format_oneline(&self, size: usize) -> String {
        match self {
            Self::Json(json) => format!("Payload({size:>3}): {json}"),
            Self::MsgPack(msgpack, _) => format!("Payload({size:>3}): {msgpack}"),
            Self::NotUtf8(err) => format!("Payload({size:>3}) is not valid UTF-8: {err}"),
            Self::String(str) => format!("Payload({size:>3}): {str}"),
        }
    }
}

#[test]
fn oneline_json_works() {
    let p = Payload::Json(serde_json::json!([42, false]));
    assert_eq!(p.format_oneline(666), "Payload(666): [42,false]");
}

#[test]
fn oneline_string_works() {
    let p = Payload::String("bar".into());
    assert_eq!(p.format_oneline(3), "Payload(  3): bar");
}

#[cfg(test)]
fn json_macro(json_str: &'static str) -> Option<String> {
    let payload = Payload::new(json_str.into());
    payload
        .as_optional_json()
        .map(std::string::ToString::to_string)
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
