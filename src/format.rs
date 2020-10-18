use chrono::{DateTime, TimeZone};
use rumqttc::{Publish, QoS};

pub fn format_published_packet<Tz: TimeZone>(packet: &Publish, time: &DateTime<Tz>) -> String
where
    Tz::Offset: std::fmt::Display,
{
    let qos = format_qos(&packet.qos);
    let timestamp = format_timestamp(packet.retain, time);

    let payload_size = packet.payload.len();
    let payload = format_payload(packet.payload.to_vec());

    format!(
        "{:12} {:50} QoS:{:11} Payload({:>3}): {}",
        timestamp, packet.topic, qos, payload_size, payload
    )
}

pub fn format_qos(qos: &QoS) -> String {
    format!("{:?}", qos)
}

pub fn format_timestamp<Tz: TimeZone>(retained: bool, time: &DateTime<Tz>) -> String
where
    Tz::Offset: std::fmt::Display,
{
    if retained {
        String::from("RETAINED")
    } else {
        time.format("%_H:%M:%S.%3f").to_string()
    }
}

pub fn format_payload(payload: Vec<u8>) -> String {
    String::from_utf8(payload).unwrap_or_else(|err| format!("invalid UTF8: {}", err))
}

pub fn format_payload_as_float(payload: Vec<u8>) -> Option<f64> {
    String::from_utf8(payload)
        .ok()
        .and_then(|o| o.parse::<f64>().ok())
}

#[test]
fn format_published_packet_works() {
    let time = DateTime::parse_from_rfc3339("2020-10-17T15:00:00+02:00").unwrap();
    let packet = Publish::new("foo", QoS::AtLeastOnce, "bar");
    assert_eq!(
        format_published_packet(&packet, &time),
        "15:00:00.000 foo                                                QoS:AtLeastOnce Payload(  3): bar"
    );
}

#[test]
fn format_timestamp_retained_has_no_timestamp() {
    let time = DateTime::parse_from_rfc3339("2020-10-17T15:00:00+02:00").unwrap();
    let formatted = format_timestamp(true, &time);
    assert!(formatted.contains("RETAINED"));
}

#[test]
fn format_qos_works() {
    assert_eq!("AtLeastOnce", format_qos(&QoS::AtLeastOnce));
    assert_eq!("AtMostOnce", format_qos(&QoS::AtMostOnce));
    assert_eq!("ExactlyOnce", format_qos(&QoS::ExactlyOnce));
}
