use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

#[derive(Clone, Copy)]
pub enum CursorMove {
    Absolute(usize),
    OneUp,
    OneDown,
    PageUp,
    PageDown,
}

pub const STYLE_BOLD: Style = Style::new().add_modifier(Modifier::BOLD);

pub const fn focus_color(has_focus: bool) -> Color {
    if has_focus {
        Color::LightGreen
    } else {
        Color::Gray
    }
}

/// When the column/row is inside the area, return the row relative to the area.
/// Otherwise `None` is returned.
pub const fn get_row_inside(area: Rect, column: u16, row: u16) -> Option<u16> {
    if row > area.top() && row < area.bottom() && column > area.left() && column < area.right() {
        Some(row - area.top() - 1)
    } else {
        None
    }
}

#[test]
fn row_outside() {
    let area = Rect::new(5, 5, 5, 10);
    let result = get_row_inside(area, 7, 1);
    assert_eq!(result, None);
}

#[test]
fn column_outside() {
    let area = Rect::new(5, 5, 5, 10);
    let result = get_row_inside(area, 1, 7);
    assert_eq!(result, None);
}

#[test]
fn is_inside() {
    let area = Rect::new(5, 5, 5, 10);
    let result = get_row_inside(area, 7, 10);
    assert_eq!(result, Some(4));
}

pub const fn split_area_vertically(area: Rect, height_first: u16) -> (Rect, Rect) {
    let first = Rect {
        height: height_first,
        ..area
    };
    let second = Rect {
        height: area.height.saturating_sub(height_first),
        y: area.y.saturating_add(height_first),
        ..area
    };
    (first, second)
}
