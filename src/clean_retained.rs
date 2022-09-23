use rumqttc::{Client, Connection, Event, Packet, QoS};

#[derive(Clone, Copy)]
pub enum Mode {
    Dry,
    Normal,
    Silent,
}
pub fn gather_topics_and_clean(
    client: &mut Client,
    mut connection: Connection,
    base_topic: &str,
    mode: Mode,
) {
    let mut known_topics: Vec<(String, bool)> = Vec::new();
    loop {
        match connection.iter().next() {
            Some(Ok(Event::Incoming(Packet::Publish(publish)))) => {
                if !publish.retain {
                    // First non retained means we exit
                    break;
                } else {
                    let topic = publish.topic;
                    let known = (topic.to_string(), true);
                    known_topics.push(known);
                }
            }
            Some(Ok(Event::Incoming(Packet::Disconnect))) => {
                break;
            }
            Some(Ok(_)) => {}
            Some(Err(e)) => {
                eprintln!("connection error: {e:?}");
                break;
            }
            None => {
                break;
            }
        }
    }
    let known_topics = known_topics.iter().map(|t| t).collect::<Vec<_>>();
    clean_retained(client, base_topic, known_topics, mode);
    client.disconnect().unwrap();
}

pub fn clean_retained(
    client: &mut Client,
    base_topic: &str,
    known_topics: Vec<&(String, bool)>,
    mode: Mode,
) {
    let filtered_topics = known_topics
        .iter()
        .filter_map(|(t, retained)| {
            if t.starts_with(base_topic) && *retained {
                Some(t)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let amount = filtered_topics.len();

    if !matches!(mode, Mode::Dry) {
        for topic in filtered_topics {
            client
                .publish(topic.to_string(), QoS::ExactlyOnce, true, [])
                .unwrap();
        }
    }
    match mode {
        Mode::Silent => {}
        Mode::Dry => println!("Dry run: would have cleaned {} topics", amount),
        Mode::Normal => println!("Cleaned {} topics", amount),
    }
}
