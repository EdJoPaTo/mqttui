use chrono::NaiveDateTime;
use tui_tree_widget::third_party::messagepack;
use tui_tree_widget::KeyValueTreeItem;

use crate::interactive::details::payload_view::PayloadView;
use crate::mqtt::HistoryEntry;
use crate::payload::Payload;

pub struct Point {
    pub time: NaiveDateTime,
    pub y: f64,
}

impl Point {
    pub fn parse(entry: &HistoryEntry, payload_view: &PayloadView) -> Option<Self> {
        let time = *entry.time.as_optional()?;
        let y = match &entry.payload {
            Payload::Binary(data) => data
                .get(payload_view.binary_state.selected_address().unwrap_or(0))
                .copied()
                .map(f64::from),
            Payload::Json(json) => {
                let selected = payload_view
                    .json_state
                    .selected()
                    .and_then(|selector| json.get_value_deep(selector));
                f64_from_json(selected.unwrap_or(json))
            }
            Payload::MessagePack(messagepack) => {
                let selected = payload_view
                    .indexed_tree_state
                    .selected()
                    .and_then(|selector| messagepack::get_value(messagepack, selector));
                f64_from_messagepack(selected.unwrap_or(messagepack))
            }
            Payload::String(str) => f64_from_string(str),
        }
        .filter(|y| y.is_finite())?;
        Some(Self { time, y })
    }

    #[allow(clippy::cast_precision_loss)]
    pub const fn as_graph_x(&self) -> f64 {
        self.time.and_utc().timestamp_millis() as f64
    }
}

#[allow(clippy::cast_precision_loss)]
fn f64_from_json(json: &serde_json::Value) -> Option<f64> {
    use serde_json::Value;
    match json {
        Value::Bool(true) => Some(1.0),
        Value::Bool(false) => Some(0.0),
        Value::Number(num) => num.as_f64(),
        Value::String(str) => f64_from_string(str),
        Value::Array(arr) => Some(arr.len() as f64),
        Value::Null | Value::Object(_) => None,
    }
}

#[allow(clippy::cast_precision_loss)]
fn f64_from_messagepack(messagepack: &rmpv::Value) -> Option<f64> {
    use rmpv::Value;
    match messagepack {
        Value::Boolean(true) => Some(1.0),
        Value::Boolean(false) => Some(0.0),
        Value::Integer(int) => int.as_f64(),
        Value::F32(float) => Some(f64::from(*float)),
        Value::F64(float) => Some(*float),
        Value::String(str) => str.as_str().and_then(f64_from_string),
        Value::Array(arr) => Some(arr.len() as f64),
        Value::Map(map) => Some(map.len() as f64),
        Value::Binary(_) | Value::Ext(_, _) | Value::Nil => None,
    }
}

fn f64_from_string(payload: &str) -> Option<f64> {
    payload
        .split(char::is_whitespace)
        .find(|str| !str.is_empty())? // lazy trim
        .parse::<f64>()
        .ok()
}

#[test]
fn f64_from_string_works() {
    fn test(input: &str, expected: Option<f64>) {
        let actual = f64_from_string(input);
        match (actual, expected) {
            (None, None) => {} // All fine
            (Some(actual), Some(expected)) => assert!(
                (actual - expected).abs() < 0.01,
                "Assertion failed:\n{actual} is not\n{expected}"
            ),
            _ => panic!("Assertion failed:\n{actual:?} is not\n{expected:?}"),
        }
    }

    test("", None);
    test("42", Some(42.0));
    test("12.3 °C", Some(12.3));
    test(" 2.4 °C", Some(2.4));
}

#[cfg(test)]
mod parse_tests {
    use rumqttc::QoS;

    use super::*;
    use crate::mqtt::Time;

    #[test]
    fn retained() {
        let entry = HistoryEntry {
            qos: QoS::AtMostOnce,
            time: Time::Retained,
            payload_size: 42,
            payload: Payload::unlimited(vec![]),
        };
        let point = Point::parse(&entry, &PayloadView::default());
        assert!(point.is_none());
    }

    #[test]
    fn json_number_works() {
        use serde_json::{Number, Value};
        let date = Time::datetime_example();
        let entry = HistoryEntry {
            qos: QoS::AtMostOnce,
            time: Time::Local(date),
            payload_size: 42,
            payload: Payload::Json(Value::Number(Number::from_f64(12.3).unwrap())),
        };
        let point = Point::parse(&entry, &PayloadView::default()).unwrap();
        assert_eq!(point.time, date);
        assert!((point.y - 12.3).abs() < 0.1);
    }

    #[test]
    fn messagepack_number_works() {
        let date = Time::datetime_example();
        let entry = HistoryEntry {
            qos: QoS::AtMostOnce,
            time: Time::Local(date),
            payload_size: 42,
            payload: Payload::MessagePack(rmpv::Value::F64(12.3)),
        };
        let point = Point::parse(&entry, &PayloadView::default()).unwrap();
        assert_eq!(point.time, date);
        assert!((point.y - 12.3).abs() < 0.1);
    }
}
