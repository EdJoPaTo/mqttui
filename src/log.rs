use std::thread::sleep;
use std::time::Duration;

use chrono::Local;
use rumqttc::Connection;

use crate::format;
use crate::mqtt::Time;
use crate::payload::Payload;

pub fn show(mut connection: Connection, verbose: bool) {
    for notification in connection.iter() {
        if let rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) =
            notification.expect("connection error")
        {
            break;
        }
    }
    for notification in connection.iter() {
        match notification {
            Ok(rumqttc::Event::Outgoing(outgoing)) => {
                if verbose {
                    println!("outgoing {outgoing:?}");
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
                let qos = format::qos(publish.qos);
                let topic = publish.topic;
                let size = publish.payload.len();
                let payload = Payload::unlimited(publish.payload.into());
                println!("{time:12} QoS:{qos:11} {topic:50} Payload({size:>3}): {payload}");
            }
            Ok(rumqttc::Event::Incoming(packet)) => {
                if verbose {
                    println!("incoming {packet:?}");
                }
            }
            Err(err) => {
                eprintln!("Connection Error: {err}");
                sleep(Duration::from_millis(25));
            }
        }
    }
}
