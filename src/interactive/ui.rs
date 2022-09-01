use crossterm::event::KeyEvent;
use tui::layout::Rect;
use tui::style::Color;

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

#[derive(Clone, Copy)]
pub enum Refresh {
    /// Update the TUI
    Update,
    /// Skip the update of the TUI
    Skip,
    /// Quit the TUI and return to the shell
    Quit,
}

pub const fn focus_color(has_focus: bool) -> Color {
    if has_focus {
        Color::LightGreen
    } else {
        Color::Gray
    }
}

/// When the column/row is inside the area, return the row relative to the area. Otherwise `None` is returned.
pub fn get_row_inside(area: Rect, column: u16, row: u16) -> Option<u16> {
    if row > area.top() && row < area.bottom() && column > area.left() && column < area.right() {
        Some(row - area.top() - 1)
    } else {
        None
    }
}
