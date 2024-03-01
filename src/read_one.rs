use std::thread::sleep;
use std::time::Duration;

use rumqttc::{Client, Connection};

use crate::payload::Payload;

pub fn show(client: &Client, mut connection: Connection, ignore_retained: bool, pretty: bool) {
    for notification in connection.iter() {
        if let rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) =
            notification.expect("connection error")
        {
            break;
        }
    }
    let mut done = false;
    for notification in connection.iter() {
        match notification {
            Ok(rumqttc::Event::Outgoing(outgoing)) => {
                if outgoing == rumqttc::Outgoing::Disconnect {
                    break;
                }
            }
            Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish))) => {
                if publish.dup || done {
                    continue;
                }
                if ignore_retained && publish.retain {
                    continue;
                }
                eprintln!("{}", publish.topic);
                if pretty {
                    let payload = Payload::unlimited(publish.payload.into());
                    println!("{payload:#}");
                } else {
                    use std::io::Write;
                    std::io::stdout()
                        .write_all(&publish.payload)
                        .expect("Should be able to write payload to stdout");
                };
                done = true;
                client.disconnect().unwrap();
            }
            Ok(rumqttc::Event::Incoming(_)) => {}
            Err(err) => {
                eprintln!("Connection Error: {err}");
                sleep(Duration::from_millis(25));
            }
        }
    }
}
