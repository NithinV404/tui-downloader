//! Input field widget for URL/file path entry

use crate::models::InputMode;
use crate::ui::theme::Styles;
use crate::ui::utils::truncate_text;
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

/// Render the input field widget
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Area to render in
/// * `text` - Current input text
/// * `mode` - Current input mode (Normal/Editing)
pub fn render(f: &mut Frame, area: Rect, text: &str, mode: InputMode) {
    let (border_style, title) = match mode {
        InputMode::Editing => (
            Styles::border_focused(),
            "📥 Add Download (Enter: submit │ Esc: cancel)",
        ),
        InputMode::Normal => (Styles::border(), "📥 Add Download (press 'i')"),
    };

    // Show placeholder or actual text
    let display_text = if text.is_empty() && mode == InputMode::Editing {
        Line::from(vec![Span::styled(
            "URL, magnet link, or .torrent/.metalink file path",
            Styles::text_muted(),
        )])
    } else if text.is_empty() {
        Line::from(vec![Span::styled(
            "Ready to download",
            Styles::text_muted(),
        )])
    } else {
        // Validate and colorize input
        let text_style = if is_valid_input(text) {
            Styles::success()
        } else if mode == InputMode::Editing {
            Styles::warning()
        } else {
            Styles::text()
        };

        // Truncate long URLs to prevent performance issues
        // Available width = area width - borders (2) - padding (2) = width - 4
        let max_width = area.width.saturating_sub(4) as usize;
        let display_str = if text.len() > max_width {
            let truncated = truncate_text(text, max_width.saturating_sub(3));
            format!("{}...", truncated)
        } else {
            text.to_string()
        };

        Line::from(vec![Span::styled(display_str, text_style)])
    };

    let input_field = Paragraph::new(display_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title)
            .border_style(border_style),
    );

    f.render_widget(input_field, area);
}

/// Validate input to provide visual feedback
fn is_valid_input(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }

    // Check for common valid patterns
    text.starts_with("http://")
        || text.starts_with("https://")
        || text.starts_with("ftp://")
        || text.starts_with("magnet:")
        || text.ends_with(".torrent")
        || text.ends_with(".metalink")
        || text.ends_with(".meta4")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_field_modes() {
        assert_eq!(InputMode::Normal, InputMode::Normal);
        assert_eq!(InputMode::Editing, InputMode::Editing);
    }

    #[test]
    fn test_is_valid_input() {
        assert!(is_valid_input("http://example.com/file.zip"));
        assert!(is_valid_input("https://example.com/file.zip"));
        assert!(is_valid_input("ftp://example.com/file.zip"));
        assert!(is_valid_input("magnet:?xt=urn:btih:abc123"));
        assert!(is_valid_input("/path/to/file.torrent"));
        assert!(is_valid_input("/path/to/file.metalink"));
        assert!(is_valid_input("/path/to/file.meta4"));
        assert!(!is_valid_input(""));
        assert!(!is_valid_input("invalid"));
    }
}
