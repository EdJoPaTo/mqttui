use chrono::NaiveDateTime;
use serde_json::Value as JsonValue;

use crate::mqtt::HistoryEntry;
use crate::payload::{JsonSelector, Payload};

#[allow(clippy::cast_precision_loss)]
const fn parse_time_to_chart_x(time: &NaiveDateTime) -> f64 {
    time.timestamp_millis() as f64
}
struct Point {
    time: NaiveDateTime,
    y: f64,
}

impl Point {
    fn parse(entry: &HistoryEntry, json_selector: &[JsonSelector]) -> Option<Self> {
        let time = *entry.time.as_optional()?;
        let y = match &entry.payload {
            Payload::Json(json) => {
                f64_from_json(JsonSelector::get_json(json, json_selector).unwrap_or(json))
            }
            Payload::MessagePack(messagepack) => f64_from_messagepack(
                JsonSelector::get_messagepack(messagepack, json_selector).unwrap_or(messagepack),
            ),
            Payload::NotUtf8(_) => None,
            Payload::String(str) => f64_from_string(str),
        }
        .filter(|y| y.is_finite())?;
        Some(Self { time, y })
    }

    const fn as_graph_point(&self) -> (f64, f64) {
        let x = parse_time_to_chart_x(&self.time);
        (x, self.y)
    }
}

#[allow(clippy::cast_precision_loss)]
fn f64_from_json(json: &JsonValue) -> Option<f64> {
    match json {
        JsonValue::Bool(true) => Some(1.0),
        JsonValue::Bool(false) => Some(0.0),
        JsonValue::Number(num) => num.as_f64(),
        JsonValue::String(str) => f64_from_string(str),
        JsonValue::Array(arr) => Some(arr.len() as f64),
        JsonValue::Null | JsonValue::Object(_) => None,
    }
}

#[allow(clippy::cast_precision_loss)]
fn f64_from_messagepack(messagepack: &rmpv::Value) -> Option<f64> {
    match messagepack {
        rmpv::Value::Boolean(true) => Some(1.0),
        rmpv::Value::Boolean(false) => Some(0.0),
        rmpv::Value::Integer(int) => int.as_f64(),
        rmpv::Value::F32(float) => Some(f64::from(*float)),
        rmpv::Value::F64(float) => Some(*float),
        rmpv::Value::String(str) => str.as_str().and_then(f64_from_string),
        rmpv::Value::Array(arr) => Some(arr.len() as f64),
        rmpv::Value::Map(map) => Some(map.len() as f64),
        rmpv::Value::Binary(_) | rmpv::Value::Ext(_, _) | rmpv::Value::Nil => None,
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

/// Dataset of Points showable by the graph. Ensures to create a useful graph (has at least 2 points)
pub struct GraphData {
    pub data: Vec<(f64, f64)>,
    pub first_time: NaiveDateTime,
    pub last_time: NaiveDateTime,
    pub x_max: f64,
    pub x_min: f64,
    pub y_max: f64,
    pub y_min: f64,
}

impl GraphData {
    pub fn parse(entries: &[HistoryEntry], json_selector: &[JsonSelector]) -> Option<Self> {
        let points = entries
            .iter()
            .filter_map(|entry| Point::parse(entry, json_selector))
            .collect::<Box<[_]>>();

        let [ref first, .., ref last] = *points else {
            return None;
        };

        let first_time = first.time;
        let last_time = last.time;
        let x_min = parse_time_to_chart_x(&first_time);
        let x_max = parse_time_to_chart_x(&last_time);

        let mut data = Vec::with_capacity(points.len());
        let mut y_min = first.y;
        let mut y_max = y_min;
        for point in points.iter() {
            y_min = y_min.min(point.y);
            y_max = y_max.max(point.y);
            data.push(point.as_graph_point());
        }

        Some(Self {
            data,
            first_time,
            last_time,
            x_max,
            x_min,
            y_max,
            y_min,
        })
    }
}
