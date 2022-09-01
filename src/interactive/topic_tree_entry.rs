use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui_tree_widget::TreeItem;

use crate::mqtt::Payload;

#[derive(Debug, PartialEq, Eq)]
pub struct TopicTreeEntry {
    pub leaf: String,
    pub messages: usize,
    pub last_payload: Option<Payload>,
    pub topics_below: usize,
    pub messages_below: usize,
    pub entries_below: Vec<Self>,
}

impl<'a> From<&'a TopicTreeEntry> for TreeItem<'a> {
    fn from(entry: &'a TopicTreeEntry) -> Self {
        let children = entry
            .entries_below
            .iter()
            .map(std::convert::Into::into)
            .collect::<Vec<_>>();

        let meta = match &entry.last_payload {
            Some(Payload::String(str)) => format!("= {}", str),
            Some(Payload::Json(json)) => format!("= {}", json.dump()),
            Some(Payload::NotUtf8(_)) => "Payload not UTF-8".to_string(),
            None => format!(
                "({} topics, {} messages)",
                entry.topics_below, entry.messages_below
            ),
        };

        let text = vec![Spans::from(vec![
            Span::styled(&entry.leaf, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(meta, Style::default().fg(Color::DarkGray)),
        ])];

        TreeItem::new(text, children)
    }
}

impl TopicTreeEntry {
    #[cfg(test)]
    /// Same examples as `MqttHistory::example`
    pub fn examples() -> Vec<Self> {
        vec![
            Self {
                leaf: "foo".into(),
                messages: 0,
                last_payload: None,
                topics_below: 2,
                messages_below: 2,
                entries_below: vec![
                    Self {
                        leaf: "bar".into(),
                        messages: 1,
                        last_payload: Some(Payload::new(&"D".into())),
                        entries_below: vec![],
                        topics_below: 0,
                        messages_below: 0,
                    },
                    Self {
                        leaf: "test".into(),
                        messages: 1,
                        last_payload: Some(Payload::new(&"B".into())),
                        entries_below: vec![],
                        topics_below: 0,
                        messages_below: 0,
                    },
                ],
            },
            Self {
                leaf: "test".into(),
                messages: 2,
                last_payload: Some(Payload::new(&"C".into())),
                topics_below: 0,
                messages_below: 0,
                entries_below: vec![],
            },
        ]
    }
}
