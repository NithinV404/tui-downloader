//! Tabs widget for switching between download categories

use crate::ui::theme::Styles;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, BorderType, Borders, Tabs},
};

/// Render the tabs widget
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Area to render in
/// * `current_tab` - Index of currently selected tab
/// * `tab_titles` - Slice of tab title strings
pub fn render(f: &mut Frame, area: Rect, current_tab: usize, tab_titles: Vec<&str>) {
    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("━━ Categories ━━")
                .border_style(Styles::border()),
        )
        .select(current_tab)
        .style(Styles::text())
        .highlight_style(Styles::highlight());

    f.render_widget(tabs, area);
}

/// Format tab title with count
///
/// # Arguments
/// * `name` - Base name of the tab
/// * `number` - Tab number (1-based)
/// * `count` - Number of items in this tab
///
/// # Returns
/// Formatted string like "[1] Active (3)"
pub fn format_tab_title(name: &str, number: usize, count: usize) -> String {
    format!("[{}] {} ({})", number, name, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tab_title() {
        assert_eq!(format_tab_title("Active", 1, 5), "[1] Active (5)");
        assert_eq!(format_tab_title("Queue", 2, 0), "[2] Queue (0)");
        assert_eq!(format_tab_title("Completed", 3, 10), "[3] Completed (10)");
    }
}
