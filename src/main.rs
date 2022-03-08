#![forbid(unsafe_code)]

use std::error::Error;
use std::time::Duration;

use rumqttc::{self, Client, MqttOptions, QoS};

mod clean_retained;
mod cli;
mod format;
mod interactive;
mod json_view;
mod log;
mod mqtt_history;
mod publish;
mod topic;
mod topic_view;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli::build().get_matches();

    let host = matches.value_of("Broker").unwrap();
    let port = matches
        .value_of("Port")
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap();

    let client_id = format!("mqttui-{:x}", rand::random::<u32>());
    let mut mqttoptions = MqttOptions::new(client_id, host, port);

    if let Some(password) = matches.value_of("Password") {
        let username = matches.value_of("Username").unwrap();
        mqttoptions.set_credentials(username, password);
    }

    if let Some(matches) = matches.subcommand_matches("clean-retained") {
        let timeout = Duration::from_secs_f32(matches.value_of("Timeout").unwrap().parse()?);
        mqttoptions.set_keep_alive(timeout);
    }

    let (mut client, connection) = Client::new(mqttoptions, 10);

    match matches.subcommand() {
        Some(("clean-retained", matches)) => {
            let topic = matches.value_of("Topic").unwrap();
            let dryrun = matches.is_present("dry-run");
            client.subscribe(topic, QoS::AtLeastOnce)?;
            clean_retained::clean_retained(client, connection, dryrun);
        }
        Some(("log", matches)) => {
            let verbose = matches.is_present("verbose");
            for topic in matches.values_of("Topics").unwrap() {
                client.subscribe(topic, QoS::AtLeastOnce)?;
            }
            log::show(connection, verbose);
        }
        Some(("publish", matches)) => {
            let verbose = matches.is_present("verbose");
            let retain = matches.is_present("retain");
            let topic = matches.value_of("Topic").unwrap();
            let payload = matches.value_of("Payload").unwrap();
            client.publish(topic, QoS::AtLeastOnce, retain, payload)?;
            publish::eventloop(client, connection, verbose);
        }
        Some((command, _)) => unreachable!("command is not available: {}", command),
        None => {
            let topic = matches.value_of("Topic").unwrap();
            let history =
                mqtt_history::MqttHistory::new(client.clone(), connection, topic.to_string())?;
            interactive::show(host, port, topic, &history)?;
            client.disconnect()?;
            history.join().expect("mqtt thread failed to finish");
        }
    }

    Ok(())
}
