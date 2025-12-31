use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn draw(frame: &mut Frame, area: Rect, title: &str, error: &str) {
    const STYLE: Style = Style::new().fg(Color::Black).bg(Color::Red);
    let block = Block::new()
        .border_style(STYLE)
        .borders(Borders::TOP)
        .title_alignment(Alignment::Center)
        .title(title);
    let paragraph = Paragraph::new(error)
        .style(STYLE)
        .wrap(Wrap { trim: false })
        .block(block);
    frame.render_widget(paragraph, area);
}
