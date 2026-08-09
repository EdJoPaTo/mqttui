use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::thread::{self, sleep};
use std::time::Duration;

use rumqttc::{Client, Connection, ConnectionError, QoS};

use crate::interactive::mqtt_history::MqttHistory;
use crate::mqtt::{HistoryEntry, Time};
use crate::payload::Payload;

type ConnectionErrorArc = Arc<RwLock<Option<ConnectionError>>>;
type HistoryArc = Arc<RwLock<MqttHistory>>;

pub struct MqttThread {
    client: Client,
    qos: QoS,
    connection_err: ConnectionErrorArc,
    history: HistoryArc,
}

impl MqttThread {
    pub fn new(
        client: Client,
        connection: Connection,
        subscribe_topic: Vec<String>,
        qos: QoS,
        payload_size_limit: usize,
    ) -> anyhow::Result<Self> {
        for topic in &subscribe_topic {
            client.subscribe(topic, qos)?;
        }

        let connection_err = Arc::new(RwLock::new(None));
        let history = Arc::new(RwLock::new(MqttHistory::new()));

        {
            let client = client.clone();
            let connection_err = Arc::clone(&connection_err);
            let history = Arc::clone(&history);
            thread::Builder::new()
                .name("mqtt connection".to_owned())
                .spawn(move || {
                    thread_logic(
                        client,
                        connection,
                        &subscribe_topic,
                        qos,
                        payload_size_limit,
                        &connection_err,
                        &history,
                    );
                })
                .expect("should be able to spawn a thread");
        }

        Ok(Self {
            client,
            qos,
            connection_err,
            history,
        })
    }

    pub fn has_connection_err(&self) -> Option<String> {
        self.connection_err
            .read()
            .expect("mqtt history thread panicked")
            .as_ref()
            .map(ToString::to_string)
    }

    pub fn get_history(&self) -> RwLockReadGuard<'_, MqttHistory> {
        self.history.read().expect("mqtt history thread panicked")
    }

    /// Remove from local cache
    pub fn uncache_topic_entry(&self, topic: &str, index: usize) -> Option<HistoryEntry> {
        self.history
            .write()
            .expect("mqtt history thread panicked")
            .uncache_topic_entry(topic, index)
    }

    /// Clean on broker
    pub fn clean_below(&self, topic: &str) -> anyhow::Result<()> {
        let topics = self.get_history().get_topics_below(topic);
        for topic in topics {
            self.client.publish(topic, self.qos, true, [])?;
        }
        Ok(())
    }
}

#[expect(clippy::needless_pass_by_value)]
fn thread_logic(
    client: Client,
    mut connection: Connection,
    subscribe_topic: &[String],
    qos: QoS,
    payload_size_limit: usize,
    connection_err: &ConnectionErrorArc,
    history: &HistoryArc,
) {
    for notification in connection.iter() {
        match notification {
            Ok(event) => {
                *connection_err.write().unwrap() = None;
                match event {
                    rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) => {
                        for topic in subscribe_topic {
                            client
                                .subscribe(topic, qos)
                                .expect("should be able to subscribe");
                        }
                    }
                    rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish)) => {
                        if publish.dup {
                            continue;
                        }
                        history.write().unwrap().add(
                            publish.topic,
                            HistoryEntry {
                                qos: publish.qos,
                                time: Time::new_now(publish.retain),
                                payload_size: publish.payload.len(),
                                payload: Payload::truncated(
                                    publish.payload.into(),
                                    payload_size_limit,
                                ),
                            },
                        );
                    }
                    rumqttc::Event::Outgoing(rumqttc::Outgoing::Disconnect) => {
                        break;
                    }
                    _ => {}
                }
            }
            Err(err) => {
                *connection_err.write().unwrap() = Some(err);
                sleep(Duration::from_millis(25));
            }
        }
    }
}
