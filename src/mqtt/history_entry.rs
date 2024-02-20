use rumqttc::QoS;

pub struct HistoryEntry {
    pub qos: QoS,
    pub time: crate::mqtt::Time,
    pub payload_size: usize,
    pub payload: crate::payload::Payload,
}
