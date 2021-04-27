use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use chrono::{DateTime, Local};
use rumqttc::{Connection, Publish};

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

type HistoryArc = Arc<Mutex<HashMap<String, Vec<HistoryEntry>>>>;

pub struct MqttHistory {
    handle: JoinHandle<()>,
    history: HistoryArc,
}

impl MqttHistory {
    pub fn new(mut connection: Connection) -> anyhow::Result<Self> {
        // Iterate until there is a ConnAck. When this fails it still fails in the main thread which is less messy. Happens for example when the host is wrong.
        for notification in connection.iter() {
            if let rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) =
                notification.expect("connection error")
            {
                break;
            }
        }

        let history = Arc::new(Mutex::new(HashMap::new()));

        let handle = {
            let history = Arc::clone(&history);
            thread::Builder::new()
                .name("mqtt connection".into())
                .spawn(move || thread_logic(connection, &history))?
        };

        Ok(Self { history, handle })
    }

    pub fn join(self) -> std::thread::Result<()> {
        self.handle.join()
    }

    pub fn get(&self, topic: &str) -> anyhow::Result<Option<Vec<HistoryEntry>>> {
        let history = self
            .history
            .lock()
            .map_err(|err| anyhow::anyhow!("failed to aquire lock of mqtt history: {}", err))?;
        let topic_entries = history.get(topic).map(|o| o.to_vec());

        Ok(topic_entries)
    }

    pub fn get_last(&self, topic: &str) -> anyhow::Result<Option<HistoryEntry>> {
        let history = self
            .history
            .lock()
            .map_err(|err| anyhow::anyhow!("failed to aquire lock of mqtt history: {}", err))?;
        let entry = history.get(topic).map(|o| o.last().unwrap().to_owned());

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
                topic: topic.to_owned(),
                messages: history.len(),
                last_payload: history.last().unwrap().packet.payload.to_vec(),
            });
        }
        result.sort_by_key(|o| o.topic.to_owned());
        Ok(result)
    }
}

fn thread_logic(mut connection: Connection, history: &HistoryArc) {
    for notification in connection.iter() {
        // While only writing to history on Incoming Publish locking the mutex here is still useful
        // When something panics here, it will poison the mutex and end the main process
        let mut history = history.lock().unwrap();

        match notification.expect("connection error") {
            rumqttc::Event::Incoming(packet) => {
                if let rumqttc::Packet::Publish(publish) = packet {
                    if publish.dup {
                        continue;
                    }

                    let time = Local::now();
                    let topic = &publish.topic;

                    if !history.contains_key(topic) {
                        history.insert(topic.to_owned(), Vec::new());
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
        };
    }
}
