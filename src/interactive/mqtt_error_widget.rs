use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn draw(f: &mut Frame, area: Rect, title: &str, error: &str) {
    const STYLE: Style = Style::new().fg(Color::Black).bg(Color::Red);
    let block = Block::new()
        .border_style(STYLE)
        .borders(Borders::TOP)
        .title(title);
    let paragraph = Paragraph::new(error)
        .style(STYLE)
        .wrap(Wrap { trim: false })
        .block(block);
    f.render_widget(paragraph, area);
}
