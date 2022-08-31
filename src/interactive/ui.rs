use crossterm::event::KeyEvent;
use tui::style::Color;

pub const fn focus_color(has_focus: bool) -> Color {
    if has_focus {
        Color::LightGreen
    } else {
        Color::Gray
    }
}

#[derive(Clone, Copy)]
pub enum Event {
    Key(KeyEvent),
    MouseClick { column: u16, row: u16 },
    MouseScrollUp,
    MouseScrollDown,
    Tick,
}

#[derive(Clone, Copy)]
pub enum CursorMove {
    Absolute(usize),
    RelativeUp,
    RelativeDown,
}
