use std::thread::sleep;
use std::time::Duration;

use chrono::Local;
use rumqttc::Connection;
use serde::Serialize;

use crate::format;
use crate::mqtt::Time;
use crate::payload::Payload;

#[derive(Serialize)]
struct JsonLog {
    time: Time,
    qos: u8,
    topic: String,
    size: usize,
    payload: Payload,
}

pub fn show(mut connection: Connection, json: bool, verbose: bool) {
    for notification in connection.iter() {
        match notification {
            Ok(rumqttc::Event::Outgoing(outgoing)) => {
                if verbose {
                    eprintln!("outgoing {outgoing:?}");
                }
                if outgoing == rumqttc::Outgoing::Disconnect {
                    break;
                }
            }
            Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish))) => {
                if publish.dup {
                    continue;
                }
                let time = if publish.retain {
                    Time::Retained
                } else {
                    Time::Local(Local::now().naive_local())
                };
                let topic = publish.topic;
                let size = publish.payload.len();
                let payload = Payload::unlimited(publish.payload.into());
                if json {
                    let json = serde_json::to_string(&JsonLog {
                        time,
                        qos: publish.qos as u8,
                        topic,
                        size,
                        payload,
                    })
                    .expect("Should be able to format log line as JSON");
                    println!("{json}");
                } else {
                    let qos = format::qos(publish.qos);
                    println!("{time:12} QoS:{qos:11} {topic:50} Payload({size:>3}): {payload}");
                };
            }
            Ok(rumqttc::Event::Incoming(packet)) => {
                if verbose {
                    eprintln!("incoming {packet:?}");
                }
            }
            Err(err) => {
                eprintln!("Connection Error: {err}");
                sleep(Duration::from_millis(25));
            }
        }
    }
}
