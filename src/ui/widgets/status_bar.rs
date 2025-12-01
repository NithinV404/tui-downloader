//! Status bar widget for displaying temporary status messages

use crate::ui::theme::Styles;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
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
        Span::raw(" "),
        Span::styled(message, style),
    ]);

    let widget = Paragraph::new(formatted_message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("━━ Status ━━")
                .border_style(style),
        )
        .alignment(Alignment::Center);

    f.render_widget(widget, area);
}

/// Determine the appropriate style and icon based on message content
fn determine_message_style(message: &str) -> (ratatui::style::Style, &'static str) {
    let lower = message.to_lowercase();

    if lower.contains("error") || lower.contains("failed") {
        (Styles::error(), "✗")
    } else if lower.contains("success")
        || lower.contains("added")
        || lower.contains("deleted")
        || lower.contains("purged")
    {
        (Styles::success(), "✓")
    } else if lower.contains("warning") {
        (Styles::warning(), "⚠")
    } else {
        (Styles::info(), "ℹ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::Theme;

    #[test]
    fn test_error_message_style() {
        let (style, icon) = determine_message_style("Error: download failed");
        assert_eq!(style.fg, Some(Theme::ERROR));
        assert_eq!(icon, "✗");
    }

    #[test]
    fn test_success_message_style() {
        let (style, icon) = determine_message_style("Successfully added download");
        assert_eq!(style.fg, Some(Theme::SUCCESS));
        assert_eq!(icon, "✓");
    }

    #[test]
    fn test_warning_message_style() {
        let (style, icon) = determine_message_style("Warning: low disk space");
        assert_eq!(style.fg, Some(Theme::WARNING));
        assert_eq!(icon, "⚠");
    }

    #[test]
    fn test_info_message_style() {
        let (style, icon) = determine_message_style("Download in progress");
        assert_eq!(style.fg, Some(Theme::INFO));
        assert_eq!(icon, "ℹ");
    }

    #[test]
    fn test_case_insensitive_detection() {
        let (style1, _) = determine_message_style("ERROR occurred");
        let (style2, _) = determine_message_style("error occurred");
        assert_eq!(style1.fg, style2.fg);
    }
}
