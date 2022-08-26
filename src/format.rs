use rumqttc::QoS;

use crate::mqtt::{HistoryEntry, Payload};

pub fn log_line(topic: &str, entry: HistoryEntry) -> String {
    let qos = qos(entry.qos);
    let time = entry.time.to_string();
    let size = entry.payload_size;
    let payload = match entry.payload {
        Payload::NotUtf8(err) => format!("Payload({:>3}) is not valid UTF-8: {}", size, err),
        Payload::String(str) => format!("Payload({:>3}): {}", size, str),
        Payload::Json(json) => format!("Payload({:>3}): {}", size, json.dump()),
    };
    format!("{:12} {:50} QoS:{:11} {}", time, topic, qos, payload)
}

pub fn qos(qos: QoS) -> String {
    format!("{:?}", qos)
}

#[test]
fn log_line_works() {
    let time = chrono::DateTime::parse_from_rfc3339("2020-10-17T15:00:00+02:00").unwrap();
    let mut packet = rumqttc::Publish::new("foo", QoS::AtLeastOnce, "bar");
    packet.retain = true;
    let entry = HistoryEntry::new(&packet, time.into());
    assert_eq!(
        log_line(&packet.topic, entry),
        "RETAINED     foo                                                QoS:AtLeastOnce Payload(  3): bar"
    );
}

#[test]
fn formats_qos() {
    assert_eq!("AtLeastOnce", qos(QoS::AtLeastOnce));
    assert_eq!("AtMostOnce", qos(QoS::AtMostOnce));
    assert_eq!("ExactlyOnce", qos(QoS::ExactlyOnce));
}
