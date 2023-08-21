use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::Frame;

pub fn draw<B: Backend>(f: &mut Frame<B>, area: Rect, title: &str, error: &str) {
    const STYLE: Style = Style {
        fg: Some(Color::Black),
        bg: Some(Color::Red),
        add_modifier: Modifier::empty(),
        sub_modifier: Modifier::empty(),
    };
    let block = Block::default()
        .border_style(STYLE)
        .borders(Borders::TOP)
        .title(title);
    let paragraph = Paragraph::new(error)
        .style(STYLE)
        .wrap(Wrap { trim: false })
        .block(block);
    f.render_widget(paragraph, area);
}
