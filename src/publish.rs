use crate::format;
use chrono::Local;
use rumqttc::{self, Client, Connection};

pub fn eventloop(mut client: Client, mut connection: Connection, verbose: bool) {
    for notification in connection.iter() {
        match notification.expect("connection error") {
            rumqttc::Event::Outgoing(outgoing) => {
                if verbose {
                    println!("outgoing {:?}", outgoing);
                }

                if let rumqttc::Outgoing::Disconnect = outgoing {
                    break;
                }
            }
            rumqttc::Event::Incoming(packet) => {
                if let rumqttc::Packet::PubAck(_) = packet {
                    // There was published something -> success -> disconnect
                    client.disconnect().unwrap();
                }

                match packet {
                    rumqttc::Packet::Publish(publish) => {
                        if publish.dup {
                            continue;
                        }

                        println!("{}", format::published_packet(&publish, &Local::now()));
                    }
                    _ => {
                        if verbose {
                            println!("incoming {:?}", packet);
                        }
                    }
                }
            }
        }
    }
}
