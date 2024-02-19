use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use rumqttc::{Client, Connection};

use crate::mqtt::Payload;

enum Finished {
    StillWaiting,
    Successfully,
    NonUtf8,
}

pub fn show(mut client: Client, mut connection: Connection, ignore_retained: bool) {
    for notification in connection.iter() {
        if let rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) =
            notification.expect("connection error")
        {
            break;
        }
    }
    let mut done = Finished::StillWaiting;
    for notification in connection.iter() {
        match notification {
            Ok(rumqttc::Event::Outgoing(outgoing)) => {
                if outgoing == rumqttc::Outgoing::Disconnect {
                    break;
                }
            }
            Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish))) => {
                if publish.dup || !matches!(done, Finished::StillWaiting) {
                    continue;
                }
                if ignore_retained && publish.retain {
                    continue;
                }
                eprintln!("{}", publish.topic);
                let size = publish.payload.len();
                done = match Payload::new(publish.payload.into()) {
                    Payload::Json(json) => {
                        println!("{json}");
                        Finished::Successfully
                    }
                    Payload::NotUtf8(err) => {
                        eprintln!("Payload ({size}) is not valid UTF-8: {err}");
                        Finished::NonUtf8
                    }
                    Payload::String(str) => {
                        println!("{str}");
                        Finished::Successfully
                    }
                };
                client.disconnect().unwrap();
            }
            Ok(rumqttc::Event::Incoming(_)) => {}
            Err(err) => {
                eprintln!("Connection Error: {err}");
                sleep(Duration::from_millis(25));
            }
        }
    }

    if matches!(done, Finished::NonUtf8) {
        exit(1);
    }
}
