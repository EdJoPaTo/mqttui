use tui::layout::Rect;
use tui::style::Color;

#[derive(Clone, Copy)]
pub enum CursorMove {
    Absolute(usize),
    RelativeUp,
    RelativeDown,
}

pub const fn focus_color(has_focus: bool) -> Color {
    if has_focus {
        Color::LightGreen
    } else {
        Color::Gray
    }
}

/// When the column/row is inside the area, return the row relative to the area.
/// Otherwise `None` is returned.
pub fn get_row_inside(area: Rect, column: u16, row: u16) -> Option<u16> {
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
