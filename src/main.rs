#![forbid(unsafe_code)]

use clap::Parser;
use cli::SubCommands;
use std::time::Duration;
use std::{error::Error, sync::Arc};

use rumqttc::{self, Client, ClientConfig, MqttOptions, QoS, TlsConfiguration, Transport};

mod clean_retained;
mod cli;
mod format;
mod interactive;
mod json_view;
mod log;
mod mqtt_packet;
mod noverifier;
mod publish;
mod topic;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli::Opt::parse();

    let host = matches.broker.clone();
    let port = matches.port;
    let client_id = matches
        .client_id
        .unwrap_or(format!("mqttui-{:x}", rand::random::<u32>()));

    let encryption = match (matches.encryption, matches.port) {
        (Some(encryption), _) => encryption,
        (None, 8883) => true,
        _ => false,
    };

    let mut mqttoptions = MqttOptions::new(client_id, host, port);
    mqttoptions.set_max_packet_size(usize::MAX, usize::MAX);
    if encryption {
        let certs = rustls_native_certs::load_native_certs().unwrap();
        let mut roots = rustls::RootCertStore::empty();
        for cert in certs {
            let _e = roots.add(&rustls::Certificate(cert.0));
        }
        let mut conf = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(roots)
            .with_no_client_auth();

        if matches.insecure {
            let mut danger = conf.dangerous();
            danger.set_certificate_verifier(Arc::new(noverifier::NoVerifier {}));
        }

        mqttoptions.set_transport(Transport::Tls(TlsConfiguration::Rustls(Arc::new(conf))));
    }

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
        Some(SubCommands::Log { topics, verbose }) => {
            for topic in topics {
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
