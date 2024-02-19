use rumqttc::{Client, Connection};

pub fn eventloop(mut client: Client, mut connection: Connection, verbose: bool) {
    for notification in connection.iter() {
        match notification.expect("connection error") {
            rumqttc::Event::Outgoing(outgoing) => {
                if verbose {
                    println!("outgoing {outgoing:?}");
                }

                if outgoing == rumqttc::Outgoing::Disconnect {
                    break;
                }
            }
            rumqttc::Event::Incoming(packet) => {
                if verbose {
                    println!("incoming {packet:?}");
                }

                if let rumqttc::Packet::PubAck(_) = packet {
                    // There was published something -> success -> disconnect
                    client.disconnect().unwrap();
                }
            }
        }
    }
}
