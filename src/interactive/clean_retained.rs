use ratatui::backend::Backend;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub fn draw_popup<B: Backend>(f: &mut Frame<B>, topic: &str) {
    let block = Block::new()
        .border_style(Style::new().fg(Color::Red))
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center)
        .title("Clean retained topics");
    let text = vec![
        Line::from("Clean the following topic and all relative below?"),
        Line::styled(
            topic,
            Style::new().add_modifier(Modifier::BOLD | Modifier::ITALIC),
        ),
        Line::from(""),
        Line::from("Confirm with Enter, abort with Esc"),
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
    // The order is important here. Clamp just panics on min > max which is not what is wanted.
    #[allow(clippy::manual_clamp)]
    let width = (r.width.saturating_mul(4) / 5)
        .max(60)
        .min(r.width.saturating_sub(4));
    let x = (r.width - width) / 2;
    let y = (r.height - height) / 2;
    Rect::new(x, y, width, height)
}
