#![forbid(unsafe_code)]

use clap::Parser;
use cli::SubCommands;
use std::{error::Error, time::Duration};

use rumqttc::{self, Client, MqttOptions, QoS};

mod clean_retained;
mod cli;
mod format;
mod interactive;
mod json_view;
mod log;
mod mqtt_packet;
mod publish;
mod topic;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli::Cli::parse();

    let host = matches.broker.clone();
    let port = matches.port;
    let client_id = matches
        .client_id
        .unwrap_or_else(|| format!("mqttui-{:x}", rand::random::<u32>()));

    let mut mqttoptions = MqttOptions::new(client_id, host, port);
    mqttoptions.set_max_packet_size(usize::MAX, usize::MAX);

    if let Some(password) = matches.password {
        let username = matches.username.unwrap();
        mqttoptions.set_credentials(username, password);
    }

    if let Some(SubCommands::CleanRetained { timeout, .. }) = matches.subcommands {
        mqttoptions.set_keep_alive(Duration::from_secs_f32(timeout));
    }

    let (mut client, connection) = Client::new(mqttoptions, 10);

    match matches.subcommands {
        Some(SubCommands::CleanRetained { topic, dry_run, .. }) => {
            let mode = if dry_run {
                clean_retained::Mode::Dry
            } else {
                clean_retained::Mode::Normal
            };
            client.subscribe(topic, QoS::AtLeastOnce)?;
            clean_retained::clean_retained(client, connection, mode);
        }
        Some(SubCommands::Log { topic, verbose }) => {
            for topic in topic {
                client.subscribe(topic, QoS::AtLeastOnce)?;
            }
            log::show(connection, verbose);
        }
        Some(SubCommands::Publish {
            topic,
            payload,
            retain,
            verbose,
        }) => {
            client.publish(topic, QoS::AtLeastOnce, retain, payload)?;
            publish::eventloop(client, connection, verbose);
        }
        None => {
            interactive::show(
                client.clone(),
                connection,
                &matches.broker,
                port,
                &matches.topic,
            )?;
            client.disconnect()?;
        }
    }

    Ok(())
}
