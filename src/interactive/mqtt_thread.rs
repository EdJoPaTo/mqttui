use std::collections::HashSet;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::thread::{self, sleep};
use std::time::Duration;

use chrono::Local;
use rumqttc::{Client, Connection, ConnectionError, QoS};

use crate::interactive::mqtt_history::MqttHistory;

type ConnectionErrorArc = Arc<RwLock<Option<ConnectionError>>>;
type HistoryArc = Arc<RwLock<MqttHistory>>;

/// The known topics, and if they were retained(true) or not
type KnownTopicsArc = Arc<RwLock<HashSet<(String, bool)>>>;

pub struct MqttThread {
    connection_err: ConnectionErrorArc,
    history: HistoryArc,
    known_topics: KnownTopicsArc,
    client: Client,
}

impl MqttThread {
    pub fn new(
        mut client: Client,
        mut connection: Connection,
        subscribe_topic: String,
    ) -> anyhow::Result<Self> {
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
        let known_topics = Arc::new(RwLock::new(HashSet::new()));
        {
            let client = client.clone();
            let connection_err = Arc::clone(&connection_err);
            let history = Arc::clone(&history);
            let known_topics = Arc::clone(&known_topics);
            thread::Builder::new()
                .name("mqtt connection".into())
                .spawn(move || {
                    thread_logic(
                        client,
                        connection,
                        &subscribe_topic,
                        &connection_err,
                        &history,
                        &known_topics,
                    );
                })?;
        }

        Ok(Self {
            client,
            connection_err,
            history,
            known_topics,
        })
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
    pub fn get_topics(&self) -> anyhow::Result<RwLockReadGuard<HashSet<(String, bool)>>> {
        self.known_topics
            .read()
            .map_err(|err| anyhow::anyhow!("failed to aquire lock of known_topics: {}", err))
    }
    pub fn get_mqtt_client(&self) -> &Client {
        &self.client
    }
}

fn thread_logic(
    mut client: Client,
    mut connection: Connection,
    subscribe_topic: &str,
    connection_err: &ConnectionErrorArc,
    history: &HistoryArc,
    known_topics: &KnownTopicsArc,
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
                        known_topics
                            .write()
                            .unwrap()
                            .insert((publish.topic, publish.retain));
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
