use std::process;
use std::thread::sleep;
use std::time::Duration;

use rumqttc::{Client, Connection};

use crate::cli::OnlyRetained;
use crate::payload::Payload;

pub fn show(client: &Client, mut connection: Connection, only: Option<OnlyRetained>, pretty: bool) {
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
                match (only, publish.retain) {
                    (Some(OnlyRetained::Retained), false) => process::exit(1),
                    (Some(OnlyRetained::Live), true) => continue,
                    _ => (),
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
                }
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
