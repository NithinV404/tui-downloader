//! Help popup widget showing all keybindings

use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

/// Render the help popup
pub fn render(f: &mut Frame, area: Rect, scroll_offset: usize) {
    // Calculate popup size (centered, 70% width, 80% height)
    let popup_area = centered_rect(70, 80, area);

    // Clear the area behind the popup
    f.render_widget(Clear, popup_area);

    // Create the popup block with rounded borders
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(" Help - Keyboard Shortcuts ")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Theme::INFO));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Build help content
    let help_content = build_help_content();
    let total_lines = help_content.len();
    let visible_lines = inner.height as usize;

    // Calculate scroll
    let max_scroll = total_lines.saturating_sub(visible_lines);
    let scroll = scroll_offset.min(max_scroll);

    // Render content
    let paragraph = Paragraph::new(help_content)
        .scroll((scroll as u16, 0))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, inner);

    // Render scrollbar if needed
    if total_lines > visible_lines {
        let scrollbar_area = Rect {
            x: popup_area.right() - 1,
            y: popup_area.y + 1,
            width: 1,
            height: popup_area.height.saturating_sub(2),
        };

        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(scroll);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"));

        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    // Render footer
    let footer_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.bottom() - 2,
        width: popup_area.width.saturating_sub(2),
        height: 1,
    };

    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Press ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" or ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            "?",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to close  |  ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            "j/k",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to scroll", Style::default().fg(Theme::TEXT_MUTED)),
    ]))
    .alignment(Alignment::Center);

    f.render_widget(footer, footer_area);
}

/// Build the help content
fn build_help_content() -> Vec<Line<'static>> {
    let mut lines = vec![];

    // Header
    lines.push(Line::from(""));
    lines.push(section_header("Navigation"));
    lines.push(key_desc("k / Up", "Move selection up"));
    lines.push(key_desc("j / Down", "Move selection down"));
    lines.push(key_desc("g / Home", "Go to first item"));
    lines.push(key_desc("G / End", "Go to last item"));
    lines.push(key_desc("Page Up / Ctrl+U", "Page up"));
    lines.push(key_desc("Page Down / Ctrl+D", "Page down"));
    lines.push(key_desc(
        "1 / 2 / 3",
        "Switch to Active/Queue/Completed tab",
    ));

    lines.push(Line::from(""));
    lines.push(section_header("Download Management"));
    lines.push(key_desc("i", "Add new download (enter URL)"));
    lines.push(key_desc("Space / p", "Pause/Resume selected download"));
    lines.push(key_desc("d", "Remove download from list"));
    lines.push(key_desc(
        "Shift+Delete",
        "Delete download AND file from disk",
    ));
    lines.push(key_desc("r", "Retry failed download"));
    lines.push(key_desc("x", "Purge all completed downloads"));
    lines.push(key_desc("Shift+P", "Pause all downloads"));
    lines.push(key_desc("Shift+R", "Resume all downloads"));

    lines.push(Line::from(""));
    lines.push(section_header("Queue Management"));
    lines.push(key_desc("Shift+K / Shift+Up", "Move download up in queue"));
    lines.push(key_desc(
        "Shift+J / Shift+Down",
        "Move download down in queue",
    ));

    lines.push(Line::from(""));
    lines.push(section_header("Search & Filter"));
    lines.push(key_desc("/", "Enter search mode"));
    lines.push(key_desc("Esc", "Clear search / Cancel"));

    lines.push(Line::from(""));
    lines.push(section_header("Sorting"));
    lines.push(key_desc(
        "s",
        "Cycle sort field (Name -> Size -> Progress -> Speed -> Status)",
    ));
    lines.push(key_desc(
        "S",
        "Toggle sort direction (Ascending/Descending)",
    ));

    lines.push(Line::from(""));
    lines.push(section_header("Speed Limits"));
    lines.push(key_desc("l", "Open speed limit settings"));

    lines.push(Line::from(""));
    lines.push(section_header("File Operations"));
    lines.push(key_desc("o", "Open downloaded file"));
    lines.push(key_desc("O", "Open containing folder"));
    lines.push(key_desc("c", "Copy download URL to clipboard"));
    lines.push(key_desc("C", "Copy file path to clipboard"));

    lines.push(Line::from(""));
    lines.push(section_header("Selection (Batch Operations)"));
    lines.push(key_desc("v", "Toggle selection on current item"));
    lines.push(key_desc("Ctrl+A", "Select all in current tab"));
    lines.push(key_desc("Ctrl+D", "Deselect all"));

    lines.push(Line::from(""));
    lines.push(section_header("Input Mode (Adding URLs)"));
    lines.push(key_desc("Enter", "Submit URL"));
    lines.push(key_desc("Esc", "Cancel input"));
    lines.push(key_desc("Ctrl+U", "Clear input line"));
    lines.push(key_desc("Ctrl+W", "Delete word backwards"));
    lines.push(key_desc("Ctrl+A / Home", "Move cursor to start"));
    lines.push(key_desc("Ctrl+E / End", "Move cursor to end"));
    lines.push(key_desc("<- / ->", "Move cursor left/right"));

    lines.push(Line::from(""));
    lines.push(section_header("General"));
    lines.push(key_desc("?", "Show this help"));
    lines.push(key_desc("F1", "Show this help"));
    lines.push(key_desc("q", "Quit application"));

    lines.push(Line::from(""));
    lines.push(section_header("Supported Formats"));
    lines.push(Line::from(vec![
        Span::styled("  * ", Style::default().fg(Theme::INFO)),
        Span::styled("HTTP/HTTPS URLs: ", Style::default().fg(Theme::CMD_COLOR)),
        Span::styled(
            "https://example.com/file.zip",
            Style::default().fg(Theme::TEXT_MUTED),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  * ", Style::default().fg(Theme::INFO)),
        Span::styled("Magnet links: ", Style::default().fg(Theme::CMD_COLOR)),
        Span::styled(
            "magnet:?xt=urn:btih:...",
            Style::default().fg(Theme::TEXT_MUTED),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  * ", Style::default().fg(Theme::INFO)),
        Span::styled("Torrent files: ", Style::default().fg(Theme::CMD_COLOR)),
        Span::styled(
            "/path/to/file.torrent",
            Style::default().fg(Theme::TEXT_MUTED),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  * ", Style::default().fg(Theme::INFO)),
        Span::styled("Metalink files: ", Style::default().fg(Theme::CMD_COLOR)),
        Span::styled(
            "/path/to/file.metalink",
            Style::default().fg(Theme::TEXT_MUTED),
        ),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(""));

    lines
}

/// Create a section header line
fn section_header(title: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::styled("  -- ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            title,
            Style::default()
                .fg(Theme::HIGHLIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" --", Style::default().fg(Theme::TEXT_MUTED)),
    ])
}

/// Create a key-description line
fn key_desc(key: &'static str, desc: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::raw("    "),
        Span::styled(
            format!("{:20}", key),
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(desc, Style::default().fg(Theme::CMD_COLOR)),
    ])
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
    fn test_help_content_not_empty() {
        let content = build_help_content();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(50, 50, area);

        // Should be roughly centered
        assert!(centered.x >= 20 && centered.x <= 30);
        assert!(centered.y >= 20 && centered.y <= 30);
    }
}
