//! Input field widget for URL/file path entry

use crate::models::InputMode;
use crate::ui::theme::Theme;
use crate::ui::utils::truncate_text;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the input field widget
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Area to render in
/// * `text` - Current input text
/// * `mode` - Current input mode (Normal/Editing)
pub fn render(f: &mut Frame, area: Rect, text: &str, mode: InputMode) {
    let is_editing = mode == InputMode::Editing;

    let border_style = if is_editing {
        Style::default().fg(Theme::BORDER_FOCUSED)
    } else {
        Style::default().fg(Theme::BORDER)
    };

    let prefix = if is_editing { ">> " } else { "   " };

    // Show placeholder or actual text
    let display_text = if text.is_empty() && is_editing {
        Line::from(vec![
            Span::styled(
                prefix,
                Style::default()
                    .fg(Theme::HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "URL, magnet link, or .torrent/.metalink file path",
                Style::default().fg(Theme::TEXT_MUTED),
            ),
            Span::styled(
                "_",
                Style::default()
                    .fg(Theme::HIGHLIGHT)
                    .add_modifier(Modifier::SLOW_BLINK),
            ), // Cursor
        ])
    } else if text.is_empty() {
        Line::from(vec![
            Span::styled(prefix, Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled("Press ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(
                "i",
                Style::default()
                    .fg(Theme::HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to add a download", Style::default().fg(Theme::TEXT_MUTED)),
        ])
    } else {
        // Validate and colorize input
        let text_style = if is_valid_input(text) {
            Style::default().fg(Theme::SUCCESS)
        } else if is_editing {
            Style::default().fg(Theme::WARNING)
        } else {
            Style::default().fg(Theme::CMD_COLOR)
        };

        // Truncate long URLs to prevent performance issues
        // Available width = area width - borders (2) - prefix (3) - cursor (1) = width - 6
        let max_width = area.width.saturating_sub(7) as usize;
        let display_str = if text.len() > max_width {
            let truncated = truncate_text(text, max_width.saturating_sub(3));
            format!("{}...", truncated)
        } else {
            text.to_string()
        };

        let mut spans = vec![
            Span::styled(
                prefix,
                if is_editing {
                    Style::default()
                        .fg(Theme::HIGHLIGHT)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Theme::TEXT_MUTED)
                },
            ),
            Span::styled(display_str, text_style),
        ];

        // Add cursor when editing
        if is_editing {
            spans.push(Span::styled(
                "_",
                Style::default()
                    .fg(Theme::HIGHLIGHT)
                    .add_modifier(Modifier::SLOW_BLINK),
            ));
        }

        Line::from(spans)
    };

    let title = if is_editing {
        " Add Download [Enter: submit | Esc: cancel] "
    } else {
        " Add Download "
    };

    let input_field = Paragraph::new(display_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
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
