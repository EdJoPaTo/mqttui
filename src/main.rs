use rumqttc::{self, Client, MqttOptions, QoS};

mod cli;
mod format;
mod simple;

fn main() {
    let args = cli::get_runtime_arguments();

    let client_id = format!("mqtt-cli-{:x}", rand::random::<u32>());
    let mut mqttoptions = MqttOptions::new(client_id, args.host, args.port);
    mqttoptions.set_keep_alive(5);

    let (mut client, mut connection) = Client::new(mqttoptions, 10);

    match args.value {
        Some(payload) => {
            simple::publish(
                &mut client,
                &mut connection,
                &args.topic,
                &payload,
                args.verbose,
            );
        }
        None => {
            simple::subscribe(
                &mut client,
                &mut connection,
                &args.topic,
                QoS::ExactlyOnce,
                args.verbose,
            );
        }
    }
}
