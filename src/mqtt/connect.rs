use std::time::Duration;

use anyhow::Context as _;
use rumqttc::{Client, Connection, Event, MqttOptions, Packet, Transport};

use crate::cli::{Broker, MqttConnection};

pub fn connect(
    MqttConnection {
        broker,
        username,
        password,
        client_id,
        client_cert,
        client_private_key,
        ca_cert,
        insecure,
    }: MqttConnection,
    keep_alive: Option<Duration>,
) -> anyhow::Result<(Broker, Client, Connection)> {
    let (transport, host, port) = match &broker {
        Broker::Tcp { host, port } => (Transport::Tcp, host.clone(), *port),
        Broker::Ssl { host, port } => (
            Transport::Tls(super::encryption::create_tls_configuration(
                insecure,
                client_cert.as_deref(),
                client_private_key.as_deref(),
                ca_cert.as_deref(),
            )?),
            host.clone(),
            *port,
        ),
        // On WebSockets the port is ignored. See https://github.com/bytebeamio/rumqtt/issues/270
        Broker::WebSocket(url) => (Transport::Ws, url.to_string(), 666),
        Broker::WebSocketSsl(url) => (
            Transport::Wss(super::encryption::create_tls_configuration(
                insecure,
                client_cert.as_deref(),
                client_private_key.as_deref(),
                ca_cert.as_deref(),
            )?),
            url.to_string(),
            666,
        ),
    };

    let client_id = client_id.unwrap_or_else(|| format!("mqttui-{:x}", rand::random::<u32>()));

    let mut mqttoptions = MqttOptions::new(client_id, host, port);
    mqttoptions.set_max_packet_size(usize::MAX, usize::MAX);
    mqttoptions.set_transport(transport);

    if let (Some(username), Some(password)) = (username, password) {
        mqttoptions.set_credentials(username, password);
    }
    if let Some(keep_alive) = keep_alive {
        mqttoptions.set_keep_alive(keep_alive);
    }

    let (client, mut connection) = Client::new(mqttoptions, 10);

    for event in connection.iter() {
        let event = event.with_context(|| format!(
            "Failed to connect to the MQTT broker {broker}.\nAre your MQTT connection options correct? For more information on them see --help"
        ))?;
        match event {
            Event::Incoming(Packet::ConnAck(_)) => return Ok((broker, client, connection)),
            Event::Incoming(packet) => eprintln!(
                "Received an MQTT packet before the ConnAck. This is suspicious behaviour of the broker {broker}. The packet: {packet:?}"
            ),
            Event::Outgoing(_) => {} // Sending stuff is fine
        }
    }
    Err(anyhow::anyhow!(
        "The MQTT connection to {broker} ended unexpectedly before it was acknowledged."
    ))
}
