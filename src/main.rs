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
#[cfg(feature = "tls")]
mod mqtt_encryption;
mod mqtt_packet;
mod publish;
mod topic;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli::Cli::parse();

    let (mut client, connection) = {
        let url = &matches.broker;

        macro_rules! only_one_of {
            ($expr1:expr, $expr2:expr, $error:literal) => {
                match ($expr1, $expr2) {
                    (None, None) => None,
                    (Some(val), None) => Some(val),
                    (None, Some(val)) => Some(val),
                    (Some(_), Some(_)) => {
                        let error = $error;
                        panic!("{error} is defined in both url and by cli flags, choose one!");
                    }
                }
            };
        }

        let (transport, host, port) = match url.scheme() {
            "mqtt" => {
                let transport = Transport::Tcp;
                let host = url.host_str().expect("Broker requires a Host").to_string();
                let port = only_one_of!(
                    url.port(),
                    matches.port,
                    "Port is defined in both url and by cli flags, choose one!"
                )
                .unwrap_or(1883);
                (transport, host, port)
            }
            #[cfg(feature = "tls")]
            "mqtts" => {
                let transport =
                    Transport::Tls(mqtt_encryption::create_tls_configuration(matches.insecure));
                let host = url.host_str().expect("Broker requires a Host").to_string();
                let port = only_one_of!(
                    url.port(),
                    matches.port,
                    "Port is defined in both url and by cli flags, choose one!"
                )
                .unwrap_or(8883);
                (transport, host, port)
            }
            // On WebSockets the port is ignored. See https://github.com/bytebeamio/rumqtt/issues/270
            #[cfg(feature = "tls")]
            "ws" => (Transport::Ws, url.to_string(), 666),
            #[cfg(feature = "tls")]
            "wss" => (
                Transport::Wss(mqtt_encryption::create_tls_configuration(matches.insecure)),
                url.to_string(),
                666,
            ),
            _ => panic!("URL scheme is not supported: {}", url.scheme()),
        };

        let mut queries: HashMap<String, String> = url
            .query_pairs()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        let client_id = only_one_of!(queries.remove("client_id"), matches.client_id, "Client ID");
        let client_id = client_id.unwrap_or(format!("mqttui-{:x}", rand::random::<u32>()));

        let mut mqttoptions = MqttOptions::new(client_id, host, port);
        mqttoptions.set_max_packet_size(usize::MAX, usize::MAX);
        mqttoptions.set_transport(transport);

        // We want the option to supply username both by url, and by flags, thus we override the url username in the case of an empty username
        let url_username = if url.username() == "" {
            None
        } else {
            Some(url.username())
        };
        let password = only_one_of!(url.password(), matches.password.as_deref(), "Password");
        if let Some(password) = password {
            let username =
                only_one_of!(url_username, matches.username.as_deref(), "Username").unwrap_or("");
            mqttoptions.set_credentials(username, password);
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
