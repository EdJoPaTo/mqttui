use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::cli::Broker;
use crate::interactive::ElementInFocus;

const VERSION_TEXT: &str = concat!("mqttui ", env!("CARGO_PKG_VERSION"));

pub struct Footer {
    broker: String,
}

impl Footer {
    pub fn new(broker: &Broker) -> Self {
        Self {
            broker: format!("{broker}"),
        }
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, focus: &ElementInFocus) {
        const STYLE: Style = Style::new()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD);
        let line = Line::from(match focus {
            ElementInFocus::TopicOverview => vec![
                Span::styled("q", STYLE),
                Span::raw(" Quit  "),
                Span::styled("Tab", STYLE),
                Span::raw(" Switch to JSON Payload  "),
                Span::styled("Del", STYLE),
                Span::raw(" Clean retained  "),
            ],
            ElementInFocus::JsonPayload => vec![
                Span::styled("q", STYLE),
                Span::raw(" Quit  "),
                Span::styled("Tab", STYLE),
                Span::raw(" Switch to Topics  "),
            ],
            ElementInFocus::CleanRetainedPopup(_) => vec![
                Span::styled("Enter", STYLE),
                Span::raw(" Clean topic tree  "),
                Span::styled("Any", STYLE),
                Span::raw(" Abort  "),
            ],
        });
        let remaining = area.width as usize - line.width();
        let full_info = format!("{VERSION_TEXT} @ {}", self.broker);
        if remaining > full_info.len() {
            let paragraph = Paragraph::new(full_info);
            f.render_widget(paragraph.alignment(Alignment::Right), area);
        } else if remaining > self.broker.len() {
            let paragraph = Paragraph::new(&*self.broker);
            f.render_widget(paragraph.alignment(Alignment::Right), area);
        } else if remaining > VERSION_TEXT.len() {
            let paragraph = Paragraph::new(VERSION_TEXT);
            f.render_widget(paragraph.alignment(Alignment::Right), area);
        }
        f.render_widget(Paragraph::new(line), area);
    }
}
