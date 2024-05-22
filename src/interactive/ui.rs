use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Borders;

pub const BORDERS_TOP_RIGHT: Borders = Borders::TOP.union(Borders::RIGHT);
pub const STYLE_BOLD: Style = Style::new().add_modifier(Modifier::BOLD);

pub enum ElementInFocus {
    TopicOverview,
    TopicSearch,
    Payload,
    HistoryTable,
    CleanRetainedPopup(String),
}

pub const fn focus_color(has_focus: bool) -> Color {
    if has_focus {
        Color::LightGreen
    } else {
        Color::Gray
    }
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

#[test]
pub fn split_vertically_example() {
    let area = Rect::new(5, 10, 10, 14);
    let (first, second) = split_area_vertically(area, 7);
    assert_eq!(first, Rect::new(5, 10, 10, 7));
    assert_eq!(second, Rect::new(5, 17, 10, 7));
}
