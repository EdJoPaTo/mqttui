#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Json(serde_json::Value),
    NotUtf8(std::str::Utf8Error),
    String(Box<str>),
}

impl Payload {
    pub fn new(payload: Vec<u8>) -> Self {
        match String::from_utf8(payload) {
            Ok(str) => {
                serde_json::from_str(&str).map_or_else(|_| Self::String(str.into()), Self::Json)
            }
            Err(err) => Self::NotUtf8(err.utf8_error()),
        }
    }

    pub fn format_oneline(&self, size: usize) -> String {
        match self {
            Self::Json(json) => format!("Payload({size:>3}): {json}"),
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
