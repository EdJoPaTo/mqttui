use chrono::NaiveDateTime;
use rumqttc::QoS;

use crate::mqtt::Payload;

#[derive(Debug, Clone, Copy)]
pub enum Time {
    Retained,
    Local(NaiveDateTime),
}

impl Time {
    pub fn new_now(retain: bool) -> Self {
        if retain {
            Self::Retained
        } else {
            Self::Local(chrono::Local::now().naive_local())
        }
    }

    pub const fn as_optional(&self) -> Option<&NaiveDateTime> {
        if let Self::Local(time) = self {
            Some(time)
        } else {
            None
        }
    }
}

impl ToString for Time {
    fn to_string(&self) -> String {
        match self {
            Self::Retained => String::from("RETAINED"),
            Self::Local(time) => time.format("%_H:%M:%S.%3f").to_string(),
        }
    }
}

pub struct HistoryEntry {
    pub qos: QoS,
    pub time: Time,
    pub payload_size: usize,
    pub payload: Payload,
}

#[test]
fn time_optional_retained() {
    let time = Time::Retained;
    assert_eq!(time.as_optional(), None);
}

#[test]
fn time_optional_time() {
    let date = chrono::NaiveDate::from_ymd_opt(1996, 12, 19)
        .unwrap()
        .and_hms_opt(16, 39, 57)
        .unwrap();
    let time = Time::Local(date);
    assert_eq!(time.as_optional(), Some(&date));
}

#[test]
fn time_retained_to_string() {
    let time = Time::Retained;
    assert_eq!(time.to_string(), "RETAINED");
}
