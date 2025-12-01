//! Popup/Modal widget for confirmations and warnings

use crate::ui::theme::Styles;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

/// Type of popup to display
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum PopupType {
    Confirmation,
    Warning,
    Error,
    Info,
}

/// Render a popup/modal dialog
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Full screen area
/// * `title` - Popup title
/// * `message` - Message to display
/// * `popup_type` - Type of popup (affects styling)
/// * `show_buttons` - Whether to show confirmation buttons
pub fn render(
    f: &mut Frame,
    area: Rect,
    title: &str,
    message: &str,
    popup_type: PopupType,
    show_buttons: bool,
) {
    // Calculate popup size (centered, 60% width, auto height)
    let popup_area = centered_rect(60, 40, area);

    // Clear the area behind the popup
    f.render_widget(Clear, popup_area);

    // Determine colors based on popup type
    let (border_style, icon) = match popup_type {
        PopupType::Confirmation => (Styles::warning(), "⚠"),
        PopupType::Warning => (Styles::warning(), "⚠"),
        PopupType::Error => (Styles::error(), "✗"),
        PopupType::Info => (Styles::info(), "ℹ"),
    };

    // Create the popup block
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(format!(" {} {} ", icon, title))
        .border_style(border_style);

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Layout: Message + Buttons
    let layout = if show_buttons {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Message area
                Constraint::Length(3), // Buttons
            ])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3)])
            .split(inner)
    };

    // Render message
    let message_lines: Vec<Line> = message
        .lines()
        .map(|line| Line::from(vec![Span::styled(line, Styles::text())]))
        .collect();

    let message_paragraph = Paragraph::new(message_lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(message_paragraph, layout[0]);

    // Render buttons if needed
    if show_buttons {
        let buttons = Line::from(vec![
            Span::styled("[", Styles::text_muted()),
            Span::styled("Y", Styles::success()),
            Span::styled("]", Styles::text_muted()),
            Span::styled(" Yes  ", Styles::text()),
            Span::styled("[", Styles::text_muted()),
            Span::styled("N", Styles::error()),
            Span::styled("]", Styles::text_muted()),
            Span::styled(" No  ", Styles::text()),
            Span::styled("[", Styles::text_muted()),
            Span::styled("Esc", Styles::text_muted()),
            Span::styled("]", Styles::text_muted()),
            Span::styled(" Cancel", Styles::text()),
        ]);

        let buttons_paragraph = Paragraph::new(buttons).alignment(Alignment::Center);
        f.render_widget(buttons_paragraph, layout[1]);
    }
}

/// Render a size warning overlay
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Full screen area
/// * `min_width` - Minimum required width
/// * `min_height` - Minimum required height
/// * `current_width` - Current terminal width
/// * `current_height` - Current terminal height
pub fn render_size_warning(
    f: &mut Frame,
    area: Rect,
    min_width: u16,
    min_height: u16,
    current_width: u16,
    current_height: u16,
) {
    // Create a centered warning box
    let popup_area = centered_rect(50, 30, area);

    // Clear the area
    f.render_widget(Clear, popup_area);

    // Create the warning block
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" ⚠ Terminal Too Small ")
        .border_style(Styles::warning());

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Create warning message
    let message = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Terminal size is too small!",
            Styles::warning(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Current: ", Styles::text_muted()),
            Span::styled(
                format!("{}x{}", current_width, current_height),
                Styles::error(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Required: ", Styles::text_muted()),
            Span::styled(format!("{}x{}", min_width, min_height), Styles::success()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Please resize your terminal",
            Styles::text_muted(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Styles::text_muted()),
            Span::styled("Q", Styles::error()),
            Span::styled(" to quit anyway", Styles::text_muted()),
        ]),
    ];

    let paragraph = Paragraph::new(message).alignment(Alignment::Center);
    f.render_widget(paragraph, inner);
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(50, 50, area);

        // Should be roughly centered
        assert!(centered.x >= 20 && centered.x <= 30);
        assert!(centered.y >= 20 && centered.y <= 30);
        assert!(centered.width >= 45 && centered.width <= 55);
        assert!(centered.height >= 45 && centered.height <= 55);
    }

    #[test]
    fn test_popup_types() {
        assert_eq!(PopupType::Confirmation, PopupType::Confirmation);
        assert_ne!(PopupType::Warning, PopupType::Error);
    }
}
