//! Shared `TestBackend` helpers for `insta` UI snapshots (SPEC §58 / §59).

use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::Frame;
use ratatui::Terminal;

/// Default full-terminal width for dialog / overlay snapshots.
pub const SNAPSHOT_WIDTH: u16 = 80;
/// Default full-terminal height for dialog / overlay snapshots.
pub const SNAPSHOT_HEIGHT: u16 = 24;

/// Row-major symbol-only ASCII for stable `insta` diffs.
pub fn buffer_snapshot_string(buf: &Buffer) -> String {
    let mut out = String::new();
    for y in buf.area.y..buf.area.y + buf.area.height {
        for x in buf.area.x..buf.area.x + buf.area.width {
            let ch = buf.get(x, y).symbol().chars().next().unwrap_or('·');
            let ch = if ch.is_control() || (ch != ' ' && !ch.is_ascii_graphic()) {
                '·'
            } else {
                ch
            };
            out.push(ch);
        }
        out.push('\n');
    }
    out
}

/// Renders a closure into a fixed-size `TestBackend` buffer.
pub fn render_to_buffer<F>(width: u16, height: u16, draw: F) -> Buffer
where
    F: FnOnce(&mut Frame),
{
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    terminal
        .draw(|f| {
            draw(f);
        })
        .expect("draw into test backend");
    terminal.backend().buffer().clone()
}

/// Full-buffer `Rect` for the given terminal dimensions.
pub fn full_area(width: u16, height: u16) -> Rect {
    Rect::new(0, 0, width, height)
}
