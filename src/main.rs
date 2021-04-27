#![forbid(unsafe_code)]

use std::error::Error;

use rumqttc::{self, Client, MqttOptions, QoS};

mod cli;
mod format;
mod interactive;
mod json_view;
mod mqtt_history;
mod publish;
mod topic;
mod topic_view;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli::build().get_matches();

    let host = matches
        .value_of("Host")
        .expect("Host could not be read from command line");

    let port = matches
        .value_of("Port")
        .and_then(|s| s.parse::<u16>().ok())
        .expect("MQTT Server Port could not be read from command line.");

    let client_id = format!("mqttui-{:x}", rand::random::<u32>());
    let mqttoptions = MqttOptions::new(client_id, host, port);
    let (mut client, connection) = Client::new(mqttoptions, 10);

    if let Some(matches) = matches.subcommand_matches("publish") {
        let verbose = matches.is_present("verbose");
        let retain = matches.is_present("retain");

        let topic = matches
            .value_of("Topic")
            .expect("Topic could not be read from command line");

        let payload = matches
            .value_of("Payload")
            .expect("Topic could not be read from command line");

        client.publish(topic, QoS::AtLeastOnce, retain, payload)?;

        publish::eventloop(client, connection, verbose);
    } else {
        let topic = matches
            .value_of("Topic")
            .expect("Topic could not be read from command line");

        client.subscribe(topic, QoS::ExactlyOnce)?;

        let history = mqtt_history::MqttHistory::new(connection)?;

        interactive::show(host, port, topic, &history)?;
        client.disconnect()?;
        history.join().expect("mqtt thread failed to finish");
    }

    Ok(())
}
