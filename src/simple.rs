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

                    let qos_formatted = format!("{:?}", publish.qos);

                    let payload_size = publish.payload.len();
                    let payload = String::from_utf8(publish.payload.to_vec())
                        .unwrap_or_else(|err| format!("invalid UTF8: {}", err));

                    let timestamp = if publish.retain {
                        String::from("RETAINED")
                    } else {
                        Local::now().format("%_H:%M:%S.%3f").to_string()
                    };
                    print!("{:12}", timestamp);

                    println!(
                        " {:50} QoS:{:11} Payload({:>3}): {}",
                        publish.topic, qos_formatted, payload_size, payload
                    )
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
