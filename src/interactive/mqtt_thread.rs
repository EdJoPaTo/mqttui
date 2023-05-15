use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::thread::{self, sleep};
use std::time::Duration;

use chrono::Local;
use rumqttc::{Client, Connection, ConnectionError, QoS};

use crate::interactive::mqtt_history::MqttHistory;

type ConnectionErrorArc = Arc<RwLock<Option<ConnectionError>>>;
type HistoryArc = Arc<RwLock<MqttHistory>>;

pub struct MqttThread {
    client: Client,
    connection_err: ConnectionErrorArc,
    history: HistoryArc,
}

impl MqttThread {
    pub fn new(
        mut client: Client,
        mut connection: Connection,
        subscribe_topic: Vec<String>,
    ) -> anyhow::Result<Self> {
        // Iterate until there is a ConnAck. When this fails it still fails in the main thread which is less messy. Happens for example when the host is wrong.
        for notification in connection.iter() {
            if let rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) = notification? {
                break;
            }
        }

        for t in &subscribe_topic {
            client.subscribe(t, QoS::ExactlyOnce)?;
        }

        let connection_err = Arc::new(RwLock::new(None));
        let history = Arc::new(RwLock::new(MqttHistory::new()));

        {
            let client = client.clone();
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
            client,
            connection_err,
            history,
        })
    }

    pub fn has_connection_err(&self) -> anyhow::Result<Option<String>> {
        match self.connection_err.read() {
            Ok(err) => Ok(err.as_ref().map(std::string::ToString::to_string)),
            Err(err) => Err(anyhow::anyhow!("mqtt history thread paniced {err}")),
        }
    }

    pub fn get_history(&self) -> anyhow::Result<RwLockReadGuard<MqttHistory>> {
        self.history
            .read()
            .map_err(|err| anyhow::anyhow!("failed to aquire lock of mqtt history: {err}"))
    }

    pub fn clean_below(&mut self, topic: &str) -> anyhow::Result<()> {
        let topics = self.get_history()?.get_topics_below(topic);
        for topic in topics {
            self.client.publish(topic, QoS::ExactlyOnce, true, [])?;
        }
        Ok(())
    }
}

fn thread_logic(
    mut client: Client,
    mut connection: Connection,
    subscribe_topic: &[String],
    connection_err: &ConnectionErrorArc,
    history: &HistoryArc,
) {
    for notification in connection.iter() {
        match notification {
            Ok(e) => {
                *connection_err.write().unwrap() = None;
                match e {
                    rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) => {
                        for t in subscribe_topic {
                            client.subscribe(t, QoS::ExactlyOnce).unwrap();
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
                *connection_err.write().unwrap() = Some(err);
                sleep(Duration::from_millis(25));
            }
        };
    }
}
