use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, sleep, JoinHandle};
use std::time::Duration;

use chrono::{DateTime, Local};
use rumqttc::{Client, Connection, ConnectionError, Publish, QoS};

pub struct TopicMessagesLastPayload {
    pub topic: String,
    pub messages: usize,
    pub last_payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub packet: Publish,
    pub time: DateTime<Local>,
}

type ConnectionErrorArc = Arc<RwLock<Option<ConnectionError>>>;
type HistoryArc = Arc<Mutex<HashMap<String, Vec<HistoryEntry>>>>;

pub struct MqttHistory {
    connection_err: ConnectionErrorArc,
    handle: JoinHandle<()>,
    history: HistoryArc,
}

impl MqttHistory {
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
        let history = Arc::new(Mutex::new(HashMap::new()));

        let handle = {
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
                    )
                })?
        };

        Ok(Self {
            connection_err,
            handle,
            history,
        })
    }

    pub fn join(self) -> std::thread::Result<()> {
        self.handle.join()
    }

    pub fn has_connection_err(&self) -> anyhow::Result<Option<String>> {
        match self.connection_err.read() {
            Ok(bla) => {
                if let Some(err) = &*bla {
                    Ok(Some(format!("{}", err)))
                } else {
                    Ok(None)
                }
            }
            Err(err) => Err(anyhow::anyhow!("mqtt history thread paniced {}", err)),
        }
    }

    pub fn get(&self, topic: &str) -> anyhow::Result<Option<Vec<HistoryEntry>>> {
        let history = self
            .history
            .lock()
            .map_err(|err| anyhow::anyhow!("failed to aquire lock of mqtt history: {}", err))?;
        let topic_entries = history.get(topic).cloned();

        Ok(topic_entries)
    }

    pub fn get_last(&self, topic: &str) -> anyhow::Result<Option<HistoryEntry>> {
        let history = self
            .history
            .lock()
            .map_err(|err| anyhow::anyhow!("failed to aquire lock of mqtt history: {}", err))?;
        let entry = history.get(topic).map(|o| o.last().unwrap().clone());

        Ok(entry)
    }

    pub fn to_tmlp(&self) -> anyhow::Result<Vec<TopicMessagesLastPayload>> {
        let history = self
            .history
            .lock()
            .map_err(|err| anyhow::anyhow!("failed to aquire lock of mqtt history: {}", err))?;
        let mut result = Vec::new();
        for (topic, history) in history.iter() {
            result.push(TopicMessagesLastPayload {
                topic: topic.clone(),
                messages: history.len(),
                last_payload: history.last().unwrap().packet.payload.to_vec(),
            });
        }
        result.sort_by_key(|o| o.topic.clone());
        Ok(result)
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
                    rumqttc::Event::Incoming(packet) => {
                        if let rumqttc::Packet::Publish(publish) = packet {
                            if publish.dup {
                                continue;
                            }

                            let time = Local::now();
                            let topic = &publish.topic;

                            let mut history = history.lock().unwrap();

                            if !history.contains_key(topic) {
                                history.insert(topic.clone(), Vec::new());
                            }

                            let vec = history.get_mut(topic).unwrap();
                            vec.push(HistoryEntry {
                                packet: publish,
                                time,
                            });
                        }
                    }
                    rumqttc::Event::Outgoing(packet) => {
                        if let rumqttc::Outgoing::Disconnect = packet {
                            break;
                        }
                    }
                }
            }
            Err(err) => {
                *connection_err = Some(err);
                drop(connection_err);
                sleep(Duration::from_millis(5000));
            }
        };
    }
}
