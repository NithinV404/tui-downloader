//! Search bar widget for filtering downloads

use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render the search bar overlay
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Full screen area
/// * `query` - Current search query
/// * `result_count` - Number of matching results
/// * `total_count` - Total number of items
pub fn render(f: &mut Frame, area: Rect, query: &str, result_count: usize, total_count: usize) {
    // Calculate search bar position (top center)
    let search_area = centered_top_rect(60, 3, area);

    // Clear the area behind the search bar
    f.render_widget(Clear, search_area);

    // Create the search bar block with rounded borders
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(" Search ")
        .border_style(Style::default().fg(Theme::HIGHLIGHT));

    let inner = block.inner(search_area);
    f.render_widget(block, search_area);

    // Use fixed layout to prevent shifting
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(10),    // Query area (flexible, takes remaining space)
            Constraint::Length(20), // Results count (fixed)
            Constraint::Length(22), // Help hints (fixed)
        ])
        .split(inner);

    // Render query section (left)
    let query_line = build_query_section(query);
    let query_para = Paragraph::new(query_line).alignment(Alignment::Left);
    f.render_widget(query_para, layout[0]);

    // Render results section (middle)
    let results_line = build_results_section(result_count, total_count);
    let results_para = Paragraph::new(results_line).alignment(Alignment::Center);
    f.render_widget(results_para, layout[1]);

    // Render hints section (right, fixed position)
    let hints_line = build_hints_section();
    let hints_para = Paragraph::new(hints_line).alignment(Alignment::Right);
    f.render_widget(hints_para, layout[2]);
}

/// Build the query section of the search bar
fn build_query_section(query: &str) -> Line<'static> {
    let mut spans = vec![];

    // Search prompt
    spans.push(Span::styled(
        ">> ",
        Style::default()
            .fg(Theme::HIGHLIGHT)
            .add_modifier(Modifier::BOLD),
    ));

    // Query text (or placeholder)
    if query.is_empty() {
        spans.push(Span::styled(
            "Type to search...",
            Style::default().fg(Theme::TEXT_MUTED),
        ));
    } else {
        spans.push(Span::styled(
            query.to_string(),
            Style::default().fg(Theme::CMD_COLOR),
        ));
        spans.push(Span::styled(
            "_",
            Style::default()
                .fg(Theme::HIGHLIGHT)
                .add_modifier(Modifier::SLOW_BLINK),
        )); // Cursor
    }

    Line::from(spans)
}

/// Build the results count section
fn build_results_section(result_count: usize, total_count: usize) -> Line<'static> {
    let mut spans = vec![];

    spans.push(Span::styled("| ", Style::default().fg(Theme::TEXT_MUTED)));

    if result_count == total_count {
        spans.push(Span::styled(
            format!("{} items", total_count),
            Style::default().fg(Theme::TEXT_MUTED),
        ));
    } else {
        spans.push(Span::styled(
            format!("{}", result_count),
            if result_count > 0 {
                Style::default().fg(Theme::SUCCESS)
            } else {
                Style::default().fg(Theme::ERROR)
            },
        ));
        spans.push(Span::styled(
            format!("/{}", total_count),
            Style::default().fg(Theme::TEXT_MUTED),
        ));
    }

    Line::from(spans)
}

/// Build the hints section
fn build_hints_section() -> Line<'static> {
    Line::from(vec![
        Span::styled(
            "Enter",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" apply ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" clear", Style::default().fg(Theme::TEXT_MUTED)),
    ])
}

/// Helper function to create a centered rectangle at the top
fn centered_top_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Small gap from top
            Constraint::Length(height),
            Constraint::Min(0), // Rest of screen
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
    fn test_centered_top_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let search_rect = centered_top_rect(50, 3, area);

        // Should be centered horizontally
        assert!(search_rect.x >= 20 && search_rect.x <= 30);
        // Should be near the top
        assert!(search_rect.y <= 5);
        // Should have correct height
        assert_eq!(search_rect.height, 3);
    }

    #[test]
    fn test_build_query_section_empty() {
        let line = build_query_section("");
        let text: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(text.contains("Type to search"));
    }

    #[test]
    fn test_build_query_section_with_query() {
        let line = build_query_section("test");
        let text: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(text.contains("test"));
        assert!(text.contains("_")); // Cursor
    }

    #[test]
    fn test_build_results_section_all_match() {
        let line = build_results_section(10, 10);
        let text: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(text.contains("10 items"));
    }

    #[test]
    fn test_build_results_section_partial_match() {
        let line = build_results_section(5, 10);
        let text: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(text.contains("5"));
        assert!(text.contains("/10"));
    }

    #[test]
    fn test_build_results_section_no_match() {
        let line = build_results_section(0, 10);
        let text: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(text.contains("0"));
    }

    #[test]
    fn test_build_hints_section() {
        let line = build_hints_section();
        let text: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(text.contains("Enter"));
        assert!(text.contains("Esc"));
    }
}
