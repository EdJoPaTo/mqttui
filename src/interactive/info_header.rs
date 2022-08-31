use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::Frame;

use crate::cli::Broker;

pub struct InfoHeader {
    broker: String,
    subscribe_topic: String,
    title: String,
}

impl InfoHeader {
    pub fn new(broker: &Broker, subscribe_topic: &str) -> Self {
        Self {
            broker: format!("MQTT Broker: {:?}", broker),
            subscribe_topic: format!("Subscribed Topic: {}", subscribe_topic),
            title: format!("MQTT TUI {}", env!("CARGO_PKG_VERSION")),
        }
    }

    pub fn draw<B>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        connection_error: Option<String>,
        selected_topic: &Option<String>,
    ) where
        B: Backend,
    {
        let mut text = vec![
            Spans::from(self.broker.clone()),
            Spans::from(self.subscribe_topic.clone()),
        ];

        if let Some(err) = connection_error {
            text.push(Spans::from(Span::styled(
                format!("MQTT Connection Error: {}", err),
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK),
            )));
        }

        if let Some(topic) = selected_topic {
            text.push(Spans::from(format!("Selected Topic: {}", topic)));
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title.clone());
        let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    }
}
