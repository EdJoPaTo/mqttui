use rumqttc::QoS;

use crate::mqtt::Payload;

pub fn payload(payload: &Payload, size: usize) -> String {
    match payload {
        Payload::NotUtf8(err) => format!("Payload({:>3}) is not valid UTF-8: {}", size, err),
        Payload::String(str) => format!("Payload({:>3}): {}", size, str),
        Payload::Json(json) => format!("Payload({:>3}): {}", size, json.dump()),
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
fn payload_works() {
    let p = Payload::String("bar".into());
    assert_eq!(payload(&p, 3), "Payload(  3): bar");
}

#[test]
fn formats_qos() {
    assert_eq!("AtLeastOnce", qos(QoS::AtLeastOnce));
    assert_eq!("AtMostOnce", qos(QoS::AtMostOnce));
    assert_eq!("ExactlyOnce", qos(QoS::ExactlyOnce));
}
