use rumqttc::{self, Client, MqttOptions, QoS};
use std::sync::Arc;

mod cli;
mod format;
mod interactive;
mod mqtt_history;
mod simple;

fn main() {
    let args = cli::get_runtime_arguments();

    let client_id = format!("mqtt-cli-{:x}", rand::random::<u32>());
    let mut mqttoptions = MqttOptions::new(client_id, &args.host, args.port);
    mqttoptions.set_keep_alive(5);

    let (mut client, connection) = Client::new(mqttoptions, 10);

    if let Some(payload) = args.value {
        client
            .publish(&args.topic, QoS::AtLeastOnce, false, payload)
            .unwrap();

        simple::eventloop(client, connection, args.verbose);
        return;
    }

    client.subscribe(&args.topic, QoS::ExactlyOnce).unwrap();

    if args.interactive {
        let (history, thread_handle) = mqtt_history::start(connection);

        interactive::show(&args.host, args.port, &args.topic, Arc::clone(&history)).unwrap();

        client.disconnect().unwrap();
        thread_handle.join().unwrap();
    } else {
        simple::eventloop(client, connection, args.verbose);
    }
}
