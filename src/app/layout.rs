//! Shared modal overlay geometry (SPEC §18.13.1 / Issue #93).

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Centered inner rectangle as a percentage of `area` (width `percent_x`, height `percent_y`).
pub(crate) fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y).div_ceil(2)),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x).div_ceil(2)),
        ])
        .split(v[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_rect_fits_inside_area() {
        let area = Rect::new(0, 0, 100, 40);
        let inner = centered_rect(area, 55, 42);
        assert!(inner.x >= area.x);
        assert!(inner.y >= area.y);
        assert!(inner.x + inner.width <= area.x + area.width);
        assert!(inner.y + inner.height <= area.y + area.height);
        assert!(inner.width > 0);
        assert!(inner.height > 0);
    }
}
