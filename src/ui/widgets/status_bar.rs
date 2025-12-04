//! Status bar widget for displaying temporary status messages

use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render the status bar widget
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Area to render in
/// * `message` - Status message to display
pub fn render(f: &mut Frame, area: Rect, message: &str) {
    if message.is_empty() {
        return;
    }

    let (style, icon) = determine_message_style(message);

    let formatted_message = Line::from(vec![
        Span::styled(icon, style),
        Span::styled(" ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(message, style),
    ]);

    let widget = Paragraph::new(formatted_message).alignment(Alignment::Center);

    f.render_widget(widget, area);
}

/// Determine the appropriate style and icon based on message content
fn determine_message_style(message: &str) -> (Style, &'static str) {
    let lower = message.to_lowercase();

    if lower.contains("error") || lower.contains("failed") {
        (
            Style::default()
                .fg(Theme::ERROR)
                .add_modifier(Modifier::BOLD),
            "[x]",
        )
    } else if lower.contains("success")
        || lower.contains("added")
        || lower.contains("deleted")
        || lower.contains("purged")
    {
        (
            Style::default()
                .fg(Theme::SUCCESS)
                .add_modifier(Modifier::BOLD),
            "[*]",
        )
    } else if lower.contains("warning") {
        (
            Style::default()
                .fg(Theme::WARNING)
                .add_modifier(Modifier::BOLD),
            "[!]",
        )
    } else {
        (Style::default().fg(Theme::INFO), "[i]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_message_style() {
        let (style, icon) = determine_message_style("Error: download failed");
        assert_eq!(style.fg, Some(Theme::ERROR));
        assert_eq!(icon, "[x]");
    }

    #[test]
    fn test_success_message_style() {
        let (style, icon) = determine_message_style("Successfully added download");
        assert_eq!(style.fg, Some(Theme::SUCCESS));
        assert_eq!(icon, "[*]");
    }

    #[test]
    fn test_warning_message_style() {
        let (style, icon) = determine_message_style("Warning: low disk space");
        assert_eq!(style.fg, Some(Theme::WARNING));
        assert_eq!(icon, "[!]");
    }

    #[test]
    fn test_info_message_style() {
        let (style, icon) = determine_message_style("Download in progress");
        assert_eq!(style.fg, Some(Theme::INFO));
        assert_eq!(icon, "[i]");
    }

    #[test]
    fn test_case_insensitive_detection() {
        let (style1, _) = determine_message_style("ERROR occurred");
        let (style2, _) = determine_message_style("error occurred");
        assert_eq!(style1.fg, style2.fg);
    }
}
