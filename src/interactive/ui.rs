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
    MouseClick(MousePosition),
    MouseScrollUp,
    MouseScrollDown,
    Tick,
}

#[derive(Clone, Copy)]
pub struct MousePosition {
    pub column: u16,
    pub row: u16,
}

#[derive(Clone, Copy)]
pub enum CursorMove {
    Absolute(usize),
    RelativeUp,
    RelativeDown,
}
