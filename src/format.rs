use rumqttc::QoS;

pub const fn qos(qos: QoS) -> &'static str {
    match qos {
        QoS::AtLeastOnce => "AtLeastOnce",
        QoS::AtMostOnce => "AtMostOnce",
        QoS::ExactlyOnce => "ExactlyOnce",
    }
}

#[test]
fn formats_qos() {
    assert_eq!("AtLeastOnce", qos(QoS::AtLeastOnce));
    assert_eq!("AtMostOnce", qos(QoS::AtMostOnce));
    assert_eq!("ExactlyOnce", qos(QoS::ExactlyOnce));
}
