use std::string::FromUtf8Error;

use chrono::{DateTime, Local};
use json::JsonValue;
use rumqttc::{Publish, QoS};

#[derive(Debug, Clone, Copy)]
pub enum Time {
    Retained,
    Local(DateTime<Local>),
}

impl Time {
    pub const fn as_optional(&self) -> Option<DateTime<Local>> {
        if let Self::Local(time) = self {
            Some(*time)
        } else {
            None
        }
    }
}

impl ToString for Time {
    fn to_string(&self) -> String {
        match self {
            // TODO: lazy_static
            Self::Retained => String::from("RETAINED"),
            Self::Local(time) => time.format("%_H:%M:%S.%3f").to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Payload {
    NotUtf8(FromUtf8Error),
    String(String),
    Json(JsonValue),
}

impl Payload {
    pub fn new(payload: &bytes::Bytes) -> Self {
        match String::from_utf8(payload.to_vec()) {
            Ok(str) => {
                if let Ok(json) = json::parse(&str) {
                    Self::Json(json)
                } else {
                    Self::String(str)
                }
            }
            Err(err) => Self::NotUtf8(err),
        }
    }

    pub const fn as_optional_json(&self) -> Option<&JsonValue> {
        if let Self::Json(json) = self {
            Some(json)
        } else {
            None
        }
    }
}

pub struct HistoryEntry {
    pub qos: QoS,
    pub time: Time,
    // TODO: move topic out?
    pub topic: String,
    pub payload_size: usize,
    pub payload: Payload,
}

impl HistoryEntry {
    pub fn new(packet: Publish, time: DateTime<Local>) -> Self {
        let time = if packet.retain {
            Time::Retained
        } else {
            Time::Local(time)
        };
        Self {
            qos: packet.qos,
            time,
            topic: packet.topic,
            payload_size: packet.payload.len(),
            payload: Payload::new(&packet.payload),
        }
    }
}

#[cfg(test)]
fn json_testcase(json_str: &'static str) -> Option<String> {
    let payload = Payload::new(&json_str.into());
    payload.as_optional_json().map(JsonValue::dump)
}

#[test]
fn payload_pretty_json_ignores_plain() {
    assert_eq!(None, json_testcase("bob"));
}

#[test]
fn payload_pretty_json_object_works() {
    assert_eq!(
        json_testcase(r#"{"a": "alpha", "b": "beta"}"#),
        Some(r#"{"a":"alpha","b":"beta"}"#.to_string())
    );
}

#[test]
fn payload_pretty_json_number_works() {
    assert_eq!(json_testcase("42"), Some("42".to_string()));
}
