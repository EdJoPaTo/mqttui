use std::thread::sleep;
use std::time::Duration;

use rumqttc::{Client, Connection, QoS};

use crate::format;
use crate::mqtt::Payload;

pub fn clean_retained(mut client: Client, mut connection: Connection, dry_run: bool) {
    let mut amount: usize = 0;
    for notification in connection.iter() {
        if let rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) =
            notification.expect("connection error")
        {
            break;
        }
    }
    for notification in connection.iter() {
        match notification {
            Ok(rumqttc::Event::Outgoing(rumqttc::Outgoing::Disconnect)) => break,
            Ok(rumqttc::Event::Outgoing(rumqttc::Outgoing::PingReq)) => {
                client.disconnect().unwrap();
            }
            Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish))) => {
                if publish.payload.is_empty() {
                    // Thats probably myself cleaning up
                    continue;
                }
                if !publish.retain {
                    client.disconnect().unwrap();
                    continue;
                }
                let topic = &publish.topic;
                {
                    let qos = format::qos(publish.qos);
                    let size = publish.payload.len();
                    let payload = format::payload(&Payload::new(publish.payload), size);
                    println!("QoS:{qos:11} {topic:50} {payload}");
                }
                amount += 1;
                if !dry_run {
                    client.publish(topic, QoS::ExactlyOnce, true, []).unwrap();
                }
            }
            Ok(_) => {}
            Err(err) => {
                eprintln!("Connection Error: {err}");
                sleep(Duration::from_millis(25));
            }
        }
    }
    if dry_run {
        println!("Dry run: would have cleaned {amount} topics");
    } else {
        println!("Cleaned {amount} topics");
    }
}
