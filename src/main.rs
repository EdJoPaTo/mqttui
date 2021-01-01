use rumqttc::{self, Client, MqttOptions, QoS};
use std::error::Error;
use std::sync::Arc;

mod cli;
mod format;
mod interactive;
mod json_view;
mod mqtt_history;
mod publish;
mod topic;
mod topic_view;

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::get_runtime_arguments();

    let client_id = format!("mqttui-{:x}", rand::random::<u32>());
    let mqttoptions = MqttOptions::new(client_id, &args.host, args.port);
    let (mut client, connection) = Client::new(mqttoptions, 10);

    if let Some(payload) = args.payload {
        client.publish(&args.topic, QoS::AtLeastOnce, false, payload)?;

        publish::eventloop(client, connection, args.verbose);
        return Ok(());
    }

    client.subscribe(&args.topic, QoS::ExactlyOnce)?;

    let (history, thread_handle) = mqtt_history::start(connection)?;

    interactive::show(&args.host, args.port, &args.topic, Arc::clone(&history))?;

    client.disconnect()?;
    thread_handle.join().expect("mqtt thread failed to finish");

    Ok(())
}
