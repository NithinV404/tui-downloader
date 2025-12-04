//! Tabs widget for switching between download categories

#![allow(dead_code)]

use crate::ui::theme::Styles;
use ratatui::{
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render the tabs widget (minimal, sleek design)
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Area to render in
/// * `current_tab` - Index of currently selected tab
/// * `tab_titles` - Slice of tab title strings
pub fn render(f: &mut Frame, area: Rect, current_tab: usize, tab_titles: Vec<&str>) {
    let mut spans = vec![];

    for (i, title) in tab_titles.iter().enumerate() {
        let is_selected = i == current_tab;

        // Add separator between tabs
        if i > 0 {
            spans.push(Span::styled("    ", Styles::text_muted()));
        }

        if is_selected {
            // Selected tab: highlighted with indicator
            spans.push(Span::styled("▸ ", Styles::highlight()));
            spans.push(Span::styled(title.to_string(), Styles::highlight()));
        } else {
            // Unselected tab: muted
            spans.push(Span::styled("  ", Styles::text_muted()));
            spans.push(Span::styled(title.to_string(), Styles::text_muted()));
        }
    }

    let tabs_line = Line::from(spans);
    let paragraph = Paragraph::new(tabs_line).alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

/// Format tab title with count
///
/// # Arguments
/// * `name` - Base name of the tab
/// * `number` - Tab number (1-based)
/// * `count` - Number of items in this tab
///
/// # Returns
/// Formatted string like "Active (3)"
pub fn format_tab_title(name: &str, number: usize, count: usize) -> String {
    format!("{} {} ({})", number, name, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tab_title() {
        assert_eq!(format_tab_title("Active", 1, 5), "1 Active (5)");
        assert_eq!(format_tab_title("Queue", 2, 0), "2 Queue (0)");
        assert_eq!(format_tab_title("Completed", 3, 10), "3 Completed (10)");
    }
}
