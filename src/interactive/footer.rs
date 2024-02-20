use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::cli::Broker;
use crate::interactive::{App, ElementInFocus};

const VERSION_TEXT: &str = concat!("mqttui ", env!("CARGO_PKG_VERSION"));

pub struct Footer {
    broker: Box<str>,
    full_info: Box<str>,
}

impl Footer {
    pub fn new(broker: &Broker) -> Self {
        Self {
            broker: format!("{broker}").into(),
            full_info: format!("{VERSION_TEXT} @ {broker}").into(),
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, app: &App) {
        const STYLE: Style = Style::new()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD);
        let line = Line::from(match app.focus {
            ElementInFocus::TopicOverview => {
                let mut result = vec![
                    Span::styled("q", STYLE),
                    Span::raw(" Quit  "),
                    Span::styled("/", STYLE),
                    Span::raw(" Search  "),
                ];
                if app.topic_overview.get_selected().is_some() {
                    result.push(Span::styled("Del", STYLE));
                    result.push(Span::raw(" Clean retained  "));
                }
                if app.can_switch_to_payload() {
                    result.push(Span::styled("Tab", STYLE));
                    result.push(Span::raw(" Switch to Payload  "));
                }
                result
            }
            ElementInFocus::TopicSearch => vec![
                Span::styled("↑", STYLE),
                Span::raw(" Before  "),
                Span::styled("↓", STYLE),
                Span::raw(" Next  "),
                Span::styled("Enter", STYLE),
                Span::raw(" Open All  "),
                Span::styled("Esc", STYLE),
                Span::raw(" Clear  "),
                Span::raw("Search: "),
                Span::raw(&app.topic_overview.search),
            ],
            ElementInFocus::Payload => vec![
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
        if remaining > self.full_info.len() {
            let paragraph = Paragraph::new(&*self.full_info);
            frame.render_widget(paragraph.alignment(Alignment::Right), area);
        } else if remaining > self.broker.len() {
            let paragraph = Paragraph::new(&*self.broker);
            frame.render_widget(paragraph.alignment(Alignment::Right), area);
        } else if remaining > VERSION_TEXT.len() {
            let paragraph = Paragraph::new(VERSION_TEXT);
            frame.render_widget(paragraph.alignment(Alignment::Right), area);
        } else {
            // Not enough space -> show nothing
        }
        frame.render_widget(Paragraph::new(line), area);
    }
}
