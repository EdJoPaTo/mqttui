use rumqttc::{self, Client, MqttOptions, QoS};

mod cli;

fn main() {
    let args = cli::get_runtime_arguments();

    let client_id = format!("quick-mqtt-cli-{:x}", rand::random::<u32>());
    let mut mqttoptions = MqttOptions::new(client_id, args.host, args.port);
    mqttoptions.set_keep_alive(5);

    let (mut client, mut connection) = Client::new(mqttoptions, 10);

    match args.value {
        Some(payload) => {
            client
                .publish(args.topic, QoS::AtLeastOnce, false, payload)
                .unwrap();
        }
        None => {
            client.subscribe(args.topic, QoS::ExactlyOnce).unwrap();
        }
    }

    let mut disconnected = false;

    for notification in connection.iter() {
        match notification {
            Err(err) => match disconnected {
                false => panic!("connection error {}", err),
                true => {
                    if args.verbose {
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

                    if args.verbose {
                        println!("outgoing {:?}", outgoing);
                    }
                }
                rumqttc::Event::Incoming(packet) => match packet {
                    rumqttc::Packet::PubAck(_) => {
                        if args.verbose {
                            println!("incoming {:?}", packet);
                        }

                        // There was published something -> success -> disconnect
                        client.disconnect().unwrap();
                    }
                    rumqttc::Packet::Publish(publish) => {
                        if publish.dup {
                            continue;
                        }

                        let qos = format!("{:?}", publish.qos);

                        let payload_size = publish.payload.len();
                        let payload = String::from_utf8(publish.payload.to_vec())
                            .unwrap_or_else(|err| format!("invalid UTF8: {}", err));

                        if publish.retain {
                            print!("{:51} RETAINED", publish.topic);
                        } else {
                            print!("{:60}", publish.topic);
                        }

                        println!(" QoS:{:11} Payload({:>3}): {}", qos, payload_size, payload)
                    }
                    _ => {
                        if args.verbose {
                            println!("incoming {:?}", packet);
                        }
                    }
                },
            },
        }
    }
}
