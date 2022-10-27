use rumqttc::QoS;

use crate::mqtt::Payload;

pub fn payload(payload: &Payload, size: usize) -> String {
    match payload {
        Payload::NotUtf8(err) => format!("Payload({size:>3}) is not valid UTF-8: {err}"),
        Payload::String(str) => format!("Payload({size:>3}): {str}"),
        Payload::Json(json) => format!("Payload({size:>3}): {}", json.dump()),
    }
}

pub const fn qos(qos: QoS) -> &'static str {
    match qos {
        QoS::AtLeastOnce => "AtLeastOnce",
        QoS::AtMostOnce => "AtMostOnce",
        QoS::ExactlyOnce => "ExactlyOnce",
    }
}

#[test]
fn payload_string_works() {
    let p = Payload::String("bar".into());
    assert_eq!(payload(&p, 3), "Payload(  3): bar");
}

#[test]
fn payload_json_works() {
    let p = Payload::Json(json::array![42, false]);
    assert_eq!(payload(&p, 666), "Payload(666): [42,false]");
}

#[test]
fn formats_qos() {
    assert_eq!("AtLeastOnce", qos(QoS::AtLeastOnce));
    assert_eq!("AtMostOnce", qos(QoS::AtMostOnce));
    assert_eq!("ExactlyOnce", qos(QoS::ExactlyOnce));
}
