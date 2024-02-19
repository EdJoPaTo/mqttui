use rumqttc::QoS;

use crate::mqtt::{Payload, Time};

pub struct HistoryEntry {
    pub qos: QoS,
    pub time: Time,
    pub payload_size: usize,
    pub payload: Payload,
}
