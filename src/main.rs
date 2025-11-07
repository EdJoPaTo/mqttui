use std::time::Duration;

use clap::Parser;
use cli::Subcommands;
use rumqttc::QoS;

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

    let keep_alive = if let Some(Subcommands::CleanRetained { timeout, .. }) = matches.subcommands {
        Some(Duration::from_secs_f32(timeout))
    } else {
        None
    };
    let (broker, client, connection) = mqtt::connect(matches.mqtt_connection, keep_alive)?;

    match matches.subcommands {
        Some(Subcommands::CleanRetained { topic, dry_run, .. }) => {
            client.subscribe(topic, QoS::AtLeastOnce)?;
            clean_retained::clean_retained(&client, connection, dry_run);
        }
        Some(Subcommands::Log {
            topic,
            json,
            verbose,
        }) => {
            for topic in topic {
                client.subscribe(topic, QoS::AtLeastOnce)?;
            }
            log::show(connection, json, verbose);
        }
        Some(Subcommands::ReadOne {
            topic,
            ignore_retained,
            mut only,
            pretty,
        }) => {
            for topic in topic {
                client.subscribe(topic, QoS::AtLeastOnce)?;
            }
            if ignore_retained {
                only = Some(cli::OnlyRetained::Live);
            }
            read_one::show(&client, connection, only, pretty);
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
            publish::eventloop(&client, connection, verbose);
        }
        None => {
            interactive::show(
                client.clone(),
                connection,
                &broker,
                matches.topic,
                matches.payload_size_limit,
            )?;
            client.disconnect()?;
        }
    }

    Ok(())
}
