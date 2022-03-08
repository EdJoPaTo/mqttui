use chrono::Local;
use rumqttc::{Client, Connection, QoS};

use crate::format;

#[derive(Clone, Copy)]
pub enum Mode {
    Dry,
    Normal,
    Silent,
}

pub fn clean_retained(mut client: Client, mut connection: Connection, mode: Mode) {
    let mut amount: usize = 0;
    for notification in connection.iter() {
        match notification.expect("connection error") {
            rumqttc::Event::Outgoing(rumqttc::Outgoing::Disconnect) => {
                break;
            }
            rumqttc::Event::Outgoing(rumqttc::Outgoing::PingReq) => {
                client.disconnect().unwrap();
            }
            rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish)) => {
                if publish.payload.is_empty() {
                    // Thats probably myself cleaning up
                    continue;
                }
                if !publish.retain {
                    client.disconnect().unwrap();
                    continue;
                }
                if !matches!(mode, Mode::Silent) {
                    println!("{}", format::published_packet(&publish, &Local::now()));
                }
                amount += 1;
                if !matches!(mode, Mode::Dry) {
                    client
                        .publish(publish.topic, QoS::ExactlyOnce, true, [])
                        .unwrap();
                }
            }
            _ => {}
        }
    }
    match mode {
        Mode::Silent => {}
        Mode::Dry => println!("Dry run: would have cleaned {} topics", amount),
        Mode::Normal => println!("Cleaned {} topics", amount),
    }
}
