use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::thread::{self, sleep};
use std::time::Duration;

use chrono::Local;
use rumqttc::{Client, Connection, ConnectionError, MqttOptions, QoS};

use crate::interactive::mqtt_history::MqttHistory;

type ConnectionErrorArc = Arc<RwLock<Option<ConnectionError>>>;
type HistoryArc = Arc<RwLock<MqttHistory>>;

pub struct MqttThread {
    connection_err: ConnectionErrorArc,
    history: HistoryArc,
    mqttoptions: MqttOptions,
}

impl MqttThread {
    pub fn new(
        mut client: Client,
        mut connection: Connection,
        subscribe_topic: String,
    ) -> anyhow::Result<Self> {
        let mqttoptions = connection.eventloop.options.clone();
        // Iterate until there is a ConnAck. When this fails it still fails in the main thread which is less messy. Happens for example when the host is wrong.
        for notification in connection.iter() {
            if let rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) =
                notification.expect("connection error")
            {
                client.subscribe(&subscribe_topic, QoS::ExactlyOnce)?;
                break;
            }
        }

        let connection_err = Arc::new(RwLock::new(None));
        let history = Arc::new(RwLock::new(MqttHistory::new()));

        {
            let connection_err = Arc::clone(&connection_err);
            let history = Arc::clone(&history);
            thread::Builder::new()
                .name("mqtt connection".into())
                .spawn(move || {
                    thread_logic(
                        client,
                        connection,
                        &subscribe_topic,
                        &connection_err,
                        &history,
                    );
                })?;
        }

        Ok(Self {
            connection_err,
            history,
            mqttoptions,
        })
    }

    pub const fn get_mqtt_options(&self) -> &MqttOptions {
        &self.mqttoptions
    }

    pub fn has_connection_err(&self) -> anyhow::Result<Option<String>> {
        match self.connection_err.read() {
            Ok(err) => Ok(err.as_ref().map(|err| format!("{}", err))),
            Err(err) => Err(anyhow::anyhow!("mqtt history thread paniced {}", err)),
        }
    }

    pub fn get_history(&self) -> anyhow::Result<RwLockReadGuard<MqttHistory>> {
        self.history
            .read()
            .map_err(|err| anyhow::anyhow!("failed to aquire lock of mqtt history: {}", err))
    }
}

fn thread_logic(
    mut client: Client,
    mut connection: Connection,
    subscribe_topic: &str,
    connection_err: &ConnectionErrorArc,
    history: &HistoryArc,
) {
    for notification in connection.iter() {
        let mut connection_err = connection_err.write().unwrap();

        match notification {
            Ok(e) => {
                *connection_err = None;

                match e {
                    rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(ack)) => {
                        if !ack.session_present {
                            client.subscribe(subscribe_topic, QoS::ExactlyOnce).unwrap();
                        }
                    }
                    rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish)) => {
                        if publish.dup {
                            continue;
                        }
                        let time = Local::now();
                        history.write().unwrap().add(&publish, time);
                    }
                    rumqttc::Event::Outgoing(rumqttc::Outgoing::Disconnect) => {
                        break;
                    }
                    _ => {}
                }
            }
            Err(err) => {
                *connection_err = Some(err);
                drop(connection_err);
                sleep(Duration::from_millis(25));
            }
        };
    }
}
