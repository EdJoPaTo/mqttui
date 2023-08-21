use tui::backend::Backend;
use tui::layout::{Alignment, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Paragraph, Wrap};
use tui::Frame;

use crate::cli::Broker;
use crate::interactive::ui::STYLE_BOLD;

pub struct InfoHeader {
    title: String,
}

impl InfoHeader {
    pub fn new(broker: &Broker) -> Self {
        Self {
            title: format!("MQTT TUI {} @ {broker}", env!("CARGO_PKG_VERSION")),
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
        let mut text = vec![Spans::from(self.title.as_str())];

        if let Some(err) = connection_error {
            const STYLE: Style = Style {
                fg: Some(Color::Red),
                bg: None,
                add_modifier: Modifier::BOLD,
                sub_modifier: Modifier::empty(),
            };
            text.push(Spans::from(Span::styled(
                format!("MQTT Connection Error: {err}"),
                STYLE,
            )));
        }

        if let Some(topic) = selected_topic {
            text.push(Spans::from(Span::styled(topic, STYLE_BOLD)));
        }

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    }
}
