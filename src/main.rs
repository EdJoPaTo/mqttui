#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::nursery)]

use std::time::Duration;

use clap::Parser;
use cli::Subcommands;
use rumqttc::{Client, MqttOptions, QoS, Transport};

mod clean_retained;
mod cli;
mod format;
mod interactive;
mod log;
mod mqtt;
mod payload;
mod publish;
mod read_one;

fn main() -> anyhow::Result<()> {
    let matches = cli::Cli::parse();

    let (mut client, connection) = {
        let (transport, host, port) = match &matches.broker {
            cli::Broker::Tcp { host, port } => (Transport::Tcp, host.clone(), *port),
            cli::Broker::Ssl { host, port } => (
                Transport::Tls(mqtt::encryption::create_tls_configuration(
                    matches.insecure,
                    &matches.client_cert,
                    &matches.client_key,
                )?),
                host.clone(),
                *port,
            ),
            // On WebSockets the port is ignored. See https://github.com/bytebeamio/rumqtt/issues/270
            cli::Broker::WebSocket(url) => (Transport::Ws, url.to_string(), 666),
            cli::Broker::WebSocketSsl(url) => (
                Transport::Wss(mqtt::encryption::create_tls_configuration(
                    matches.insecure,
                    &matches.client_cert,
                    &matches.client_key,
                )?),
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

        if let Some(Subcommands::CleanRetained { timeout, .. }) = matches.subcommands {
            mqttoptions.set_keep_alive(Duration::from_secs_f32(timeout));
        }

        Client::new(mqttoptions, 10)
    };

    match matches.subcommands {
        Some(Subcommands::CleanRetained { topic, dry_run, .. }) => {
            client.subscribe(topic, QoS::AtLeastOnce)?;
            clean_retained::clean_retained(client, connection, dry_run);
        }
        Some(Subcommands::Log { topic, verbose }) => {
            for topic in topic {
                client.subscribe(topic, QoS::AtLeastOnce)?;
            }
            log::show(connection, verbose);
        }
        Some(Subcommands::ReadOne {
            topic,
            ignore_retained,
        }) => {
            for topic in topic {
                client.subscribe(topic, QoS::AtLeastOnce)?;
            }
            read_one::show(client, connection, ignore_retained);
        }
        Some(Subcommands::Publish {
            topic,
            payload,
            retain,
            verbose,
        }) => {
            let payload = payload.map_or_else(
                || {
                    use std::io::Read;
                    let mut buffer = Vec::new();
                    std::io::stdin()
                        .read_to_end(&mut buffer)
                        .expect("Should be able to read the payload from stdin");
                    buffer
                },
                String::into_bytes,
            );
            client.publish(topic, QoS::AtLeastOnce, retain, payload)?;
            publish::eventloop(client, connection, verbose);
        }
        None => {
            let broker = matches.broker;
            interactive::show(client.clone(), connection, &broker, matches.topic)?;
            client.disconnect()?;
        }
    }

    Ok(())
}
