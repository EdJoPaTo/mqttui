use tui::backend::Backend;
use tui::layout::{Alignment, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Clear, Paragraph};
use tui::Frame;

use super::mqtt_thread::MqttThread;

pub fn draw_popup<B: Backend>(f: &mut Frame<B>, topic: &str) {
    let block = Block::default()
        .border_style(Style::default().fg(Color::Red))
        .borders(Borders::ALL)
        .title("Clean retained topics");
    let text = vec![
        Spans::from("Clean the following topic and all relative below?"),
        Spans::from(Span::styled(
            topic,
            Style::default().add_modifier(Modifier::BOLD | Modifier::ITALIC),
        )),
        Spans::from(""),
        Spans::from("Confirm with Enter, abort with Esc"),
    ];
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
    let area = popup_area(f.size());
    f.render_widget(Clear, area); // clear the background of the popup
    f.render_widget(paragraph, area);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(r: Rect) -> Rect {
    let height = 6;
    let width = (r.width.saturating_mul(4) / 5)
        .max(60)
        .min(r.width.saturating_sub(4));
    let x = (r.width - width) / 2;
    let y = (r.height - height) / 2;
    Rect::new(x, y, width, height)
}

// TODO: use mqtt_thread instead of new mqtt connection
// some people have strict requirements on their connection so a new connection wont help them here
pub fn do_clear(mqtt_thread: &MqttThread, topic: &str) -> anyhow::Result<()> {
    let known_topics = mqtt_thread.get_topics().unwrap();
    let known_topics = known_topics.iter().collect::<Vec<_>>();
    let mut client = mqtt_thread.get_mqtt_client().clone();

    crate::clean_retained::clean_retained(
        &mut client,
        topic,
        known_topics,
        crate::clean_retained::Mode::Silent,
    );

    Ok(())
}
