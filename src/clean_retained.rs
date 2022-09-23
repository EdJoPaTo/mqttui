use rumqttc::{Client, QoS};

#[derive(Clone, Copy)]
pub enum Mode {
    Dry,
    Normal,
    Silent,
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

    eprintln!("{filtered_topics:?}");
    let mut amount = filtered_topics.len();

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
