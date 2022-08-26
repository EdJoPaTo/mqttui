#![forbid(unsafe_code)]

use clap::Parser;
use cli::SubCommands;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

use rumqttc::{self, Client, MqttOptions, QoS, Transport};

mod clean_retained;
mod cli;
mod format;
mod interactive;
mod json_view;
mod log;
mod mqtt;
mod publish;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli::Cli::parse();

    let (mut client, connection) = {
        let url = &matches.broker;

        let (transport, host, port) = match url.scheme() {
            "mqtt" => (
                Transport::Tcp,
                url.host_str().expect("Broker requires a Host").to_string(),
                url.port().unwrap_or(1883),
            ),
            #[cfg(feature = "tls")]
            "mqtts" => (
                Transport::Tls(mqtt::encryption::create_tls_configuration(matches.insecure)),
                url.host_str().expect("Broker requires a Host").to_string(),
                url.port().unwrap_or(8883),
            ),
            // On WebSockets the port is ignored. See https://github.com/bytebeamio/rumqtt/issues/270
            #[cfg(feature = "tls")]
            "ws" => (Transport::Ws, url.to_string(), 666),
            #[cfg(feature = "tls")]
            "wss" => (
                Transport::Wss(mqtt::encryption::create_tls_configuration(matches.insecure)),
                url.to_string(),
                666,
            ),
            _ => panic!("URL scheme is not supported: {}", url.scheme()),
        };

        let mut queries = url.query_pairs().collect::<HashMap<_, _>>();

        let client_id = queries.remove("client_id").map_or_else(
            || format!("mqttui-{:x}", rand::random::<u32>()),
            |o| o.to_string(),
        );

        let mut mqttoptions = MqttOptions::new(client_id, host, port);
        mqttoptions.set_max_packet_size(usize::MAX, usize::MAX);
        mqttoptions.set_transport(transport);

        if let Some(password) = url.password() {
            mqttoptions.set_credentials(url.username(), password);
        }

        if let Some(SubCommands::CleanRetained { timeout, .. }) = matches.subcommands {
            mqttoptions.set_keep_alive(Duration::from_secs_f32(timeout));
        }

        assert!(
            queries.is_empty(),
            "Broker URL has superfluous query arguments: {:?}",
            queries
        );

        Client::new(mqttoptions, 10)
    };

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
            let mut display_broker = matches.broker.clone();
            display_broker.set_password(None).unwrap();
            display_broker.set_query(None);

            interactive::show(client.clone(), connection, display_broker, matches.topic)?;
            client.disconnect()?;
        }
    }

    Ok(())
}
