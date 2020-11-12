use chrono::{DateTime, TimeZone};
use json::JsonValue;
use rumqttc::{Publish, QoS};

pub const TIMESTAMP_FORMAT: &str = "%_H:%M:%S.%3f";
pub const TIMESTAMP_RETAINED: &str = "RETAINED";

pub fn published_packet<Tz: TimeZone>(packet: &Publish, time: &DateTime<Tz>) -> String
where
    Tz::Offset: std::fmt::Display,
{
    let qos = qos(packet.qos);
    let timestamp = timestamp(packet.retain, time);

    let payload_size = packet.payload.len();
    let payload = payload_as_utf8(packet.payload.to_vec());

    format!(
        "{:12} {:50} QoS:{:11} Payload({:>3}): {}",
        timestamp, packet.topic, qos, payload_size, payload
    )
}

pub fn qos(qos: QoS) -> String {
    format!("{:?}", qos)
}

pub fn timestamp<Tz: TimeZone>(retained: bool, time: &DateTime<Tz>) -> String
where
    Tz::Offset: std::fmt::Display,
{
    if retained {
        String::from(TIMESTAMP_RETAINED)
    } else {
        time.format(TIMESTAMP_FORMAT).to_string()
    }
}

pub fn payload_as_utf8(payload: Vec<u8>) -> String {
    String::from_utf8(payload).unwrap_or_else(|err| format!("invalid UTF8: {}", err))
}

pub fn payload_as_float(payload: Vec<u8>) -> Option<f64> {
    String::from_utf8(payload)
        .ok()
        .and_then(|o| o.parse::<f64>().ok())
}

pub fn payload_as_json(payload: Vec<u8>) -> Option<JsonValue> {
    String::from_utf8(payload)
        .ok()
        .and_then(|s| json::parse(&s).ok())
}

#[test]
fn formats_published_packet() {
    let time = DateTime::parse_from_rfc3339("2020-10-17T15:00:00+02:00").unwrap();
    let packet = Publish::new("foo", QoS::AtLeastOnce, "bar");
    assert_eq!(
        published_packet(&packet, &time),
        "15:00:00.000 foo                                                QoS:AtLeastOnce Payload(  3): bar"
    );
}

#[test]
fn formatted_timestamp_retained_has_no_timestamp() {
    let time = DateTime::parse_from_rfc3339("2020-10-17T15:00:00+02:00").unwrap();
    let formatted = timestamp(true, &time);
    assert!(formatted.contains("RETAINED"));
}

#[test]
fn formats_qos() {
    assert_eq!("AtLeastOnce", qos(QoS::AtLeastOnce));
    assert_eq!("AtMostOnce", qos(QoS::AtMostOnce));
    assert_eq!("ExactlyOnce", qos(QoS::ExactlyOnce));
}

#[test]
fn payload_pretty_json_ignores_plain() {
    assert_eq!(None, payload_as_json(b"bob".to_vec()))
}

#[test]
fn payload_pretty_json_object_works() {
    assert_eq!(
        payload_as_json(br#"{"a": "alpha", "b": "beta"}"#.to_vec()).map(json::stringify),
        Some(r#"{"a":"alpha","b":"beta"}"#.to_string())
    );
}

#[test]
fn payload_pretty_json_number_works() {
    assert_eq!(
        payload_as_json(b"42".to_vec()).map(json::stringify),
        Some("42".to_string())
    );
}
