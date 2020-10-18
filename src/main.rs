use rumqttc::{self, Client, MqttOptions, QoS};

mod cli;
mod format;
mod simple;

fn main() {
    let args = cli::get_runtime_arguments();

    let client_id = format!("mqtt-cli-{:x}", rand::random::<u32>());
    let mut mqttoptions = MqttOptions::new(client_id, args.host, args.port);
    mqttoptions.set_keep_alive(5);

    let (mut client, connection) = Client::new(mqttoptions, 10);

    if let Some(payload) = args.value {
        client
            .publish(&args.topic, QoS::AtLeastOnce, false, payload)
            .unwrap();
    } else {
        client.subscribe(&args.topic, QoS::ExactlyOnce).unwrap();
    }

    simple::eventloop(client, connection, args.verbose);
}
