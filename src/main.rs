#![forbid(unsafe_code)]

use clap::Parser;
use cli::SubCommands;
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
        let (transport, host, port) = match &matches.broker {
            cli::Broker::Tcp { host, port } => (Transport::Tcp, host.clone(), *port),
            #[cfg(feature = "tls")]
            cli::Broker::Ssl { host, port } => (
                Transport::Tls(mqtt::encryption::create_tls_configuration(matches.insecure)),
                host.clone(),
                *port,
            ),
            // On WebSockets the port is ignored. See https://github.com/bytebeamio/rumqtt/issues/270
            #[cfg(feature = "tls")]
            cli::Broker::WebSocket(url) => (Transport::Ws, url.to_string(), 666),
            #[cfg(feature = "tls")]
            cli::Broker::WebSocketSsl(url) => (
                Transport::Wss(mqtt::encryption::create_tls_configuration(matches.insecure)),
                url.to_string(),
                666,
            ),
        };

        let client_id = matches
            .client_id
            .unwrap_or_else(|| format!("mqttui-{:x}", rand::random::<u32>()));

        let mut mqttoptions = MqttOptions::new(client_id, host, port);
        mqttoptions.set_max_packet_size(usize::MAX, usize::MAX);
        mqttoptions.set_transport(transport);

        if let Some(password) = matches.password {
            let username = matches.username.unwrap();
            mqttoptions.set_credentials(username, password);
        }

        if let Some(SubCommands::CleanRetained { timeout, .. }) = matches.subcommands {
            mqttoptions.set_keep_alive(Duration::from_secs_f32(timeout));
        }

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
            let broker = matches.broker;
            interactive::show(client.clone(), connection, broker, matches.topic)?;
            client.disconnect()?;
        }
    }

    Ok(())
}
