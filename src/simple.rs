use crate::format::format_published_packet;
use chrono::Local;
use rumqttc::{self, Client, Connection, QoS};

pub fn publish(
    client: &mut Client,
    connection: &mut Connection,
    topic: &str,
    payload: &str,
    verbose: bool,
) {
    client
        .publish(topic, QoS::AtLeastOnce, false, payload)
        .unwrap();

    let mut disconnected = false;

    for notification in connection.iter() {
        match notification {
            Err(err) => match disconnected {
                false => panic!("connection error: {}", err),
                true => {
                    if verbose {
                        println!("disconnected: {}", err);
                    }
                    break;
                }
            },
            Ok(event) => match event {
                rumqttc::Event::Outgoing(outgoing) => {
                    if let rumqttc::Outgoing::Disconnect = outgoing {
                        disconnected = true
                    }

                    if verbose {
                        println!("outgoing {:?}", outgoing);
                    }
                }
                rumqttc::Event::Incoming(packet) => {
                    if let rumqttc::Packet::PubAck(_) = packet {
                        // There was published something -> success -> disconnect
                        client.disconnect().unwrap();
                    }

                    if verbose {
                        println!("incoming {:?}", packet);
                    }
                }
            },
        }
    }
}

pub fn subscribe(
    client: &mut Client,
    connection: &mut Connection,
    topic: &str,
    qos: QoS,
    verbose: bool,
) {
    client.subscribe(topic, qos).unwrap();

    for notification in connection.iter() {
        match notification.expect("connection error") {
            rumqttc::Event::Outgoing(outgoing) => {
                if verbose {
                    println!("outgoing {:?}", outgoing);
                }
            }
            rumqttc::Event::Incoming(packet) => match packet {
                rumqttc::Packet::Publish(publish) => {
                    if publish.dup {
                        continue;
                    }

                    println!("{}", format_published_packet(&publish, &Local::now()));
                }
                _ => {
                    if verbose {
                        println!("incoming {:?}", packet);
                    }
                }
            },
        };
    }
}
