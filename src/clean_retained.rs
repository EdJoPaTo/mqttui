use chrono::Local;
use rumqttc::{Client, Connection, QoS};

use crate::format;

pub fn clean_retained(mut client: Client, mut connection: Connection, dryrun: bool) {
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
                println!("{}", format::published_packet(&publish, &Local::now()));
                amount += 1;
                if !dryrun {
                    client
                        .publish(publish.topic, QoS::ExactlyOnce, true, [])
                        .unwrap();
                }
            }
            _ => {}
        }
    }
    if dryrun {
        println!("Dry run: would have cleaned {} topics", amount);
    } else {
        println!("Cleaned {} topics", amount);
    }
}
