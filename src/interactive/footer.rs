use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::cli::Broker;
use crate::interactive::{App, ElementInFocus};

const VERSION_TEXT: &str = concat!("mqttui ", env!("CARGO_PKG_VERSION"));
const KEY_STYLE: Style = Style::new()
    .fg(Color::Black)
    .bg(Color::Gray)
    .add_modifier(Modifier::BOLD);

macro_rules! key {
    ( $key:expr,$text:expr ) => {
        [
            Span::styled(concat![" ", $key, " "], KEY_STYLE),
            Span::raw(concat![" ", $text, " "]),
        ]
    };
}
macro_rules! addkey {
    ( $vec:expr,$key:expr,$text:expr ) => {
        let [key, text] = key! {$key, $text};
        $vec.push(key);
        $vec.push(text);
    };
}

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
        let line = Line::from(match app.focus {
            ElementInFocus::TopicOverview => {
                let mut result = [key!("q", "Quit"), key!("/", "Search")].concat();
                if app.topic_overview.get_selected().is_some() {
                    addkey!(result, "Del", "Clean retained");
                }
                if app.can_switch_to_payload() {
                    addkey!(result, "Tab", "Switch to Payload");
                } else if app.can_switch_to_history_table() {
                    addkey!(result, "Tab", "Switch to History");
                } else {
                    // Changing somewhere is pointless currently
                }
                result
            }
            ElementInFocus::TopicSearch => {
                let mut result = [
                    key!("↑", "Before"),
                    key!("↓", "Next"),
                    key!("Enter", "Open All"),
                    key!("Esc", "Clear"),
                ]
                .concat();
                result.push(Span::styled(
                    " Search: ",
                    Style::new()
                        .fg(Color::Black)
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                ));
                result.push(Span::raw(" "));
                result.push(Span::raw(&app.topic_overview.search));
                result
            }
            ElementInFocus::Payload => {
                let mut result = [key!("q", "Quit")].concat();
                if app.can_switch_to_history_table() {
                    addkey!(result, "Tab", "Switch to History");
                } else {
                    addkey!(result, "Tab", "Switch to Topics");
                }
                result
            }
            ElementInFocus::HistoryTable => {
                [key!("q", "Quit"), key!("Tab", "Switch to Topics")].concat()
            }
            ElementInFocus::CleanRetainedPopup(_) => {
                [key!("Enter", "Clean topic tree"), key!("Any", "Abort")].concat()
            }
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
