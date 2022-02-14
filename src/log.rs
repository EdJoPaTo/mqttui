use chrono::Local;
use rumqttc::Connection;

use crate::format;

pub fn show(mut connection: Connection, verbose: bool) {
    for notification in connection.iter() {
        match notification.expect("connection error") {
            rumqttc::Event::Outgoing(outgoing) => {
                if verbose {
                    println!("outgoing {:?}", outgoing);
                }
                if outgoing == rumqttc::Outgoing::Disconnect {
                    break;
                }
            }
            rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish)) => {
                if publish.dup {
                    continue;
                }
                println!("{}", format::published_packet(&publish, &Local::now()));
            }
            rumqttc::Event::Incoming(packet) => {
                if verbose {
                    println!("incoming {:?}", packet);
                }
            }
        }
    }
}
