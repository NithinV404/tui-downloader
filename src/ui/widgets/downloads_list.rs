//! Downloads list widget for displaying download items

use crate::models::Download;
use crate::ui::theme::Styles;
use crate::ui::utils::truncate_text;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, LineGauge, ListState, Paragraph},
};

/// Render the downloads list widget
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Area to render in
/// * `downloads` - Slice of downloads to display
/// * `list_state` - Mutable list state for selection tracking
pub fn render(f: &mut Frame, area: Rect, downloads: &[&Download], list_state: &mut ListState) {
    // Validate and adjust list state
    validate_selection(list_state, downloads.len());

    let selected_index = list_state.selected().unwrap_or(0);

    // Calculate scrolling
    let visible_rows = (area.height as usize).saturating_sub(2); // -2 for borders
    let items_per_screen = visible_rows / 2; // Each item takes 2 rows
    let scroll_offset = calculate_scroll_offset(selected_index, items_per_screen);

    // Render items
    let mut current_y = area.top() + 1; // +1 for top border

    for (idx, download) in downloads.iter().enumerate() {
        // Skip items before scroll offset
        if idx < scroll_offset {
            continue;
        }

        // Stop if we've filled the visible area
        if current_y + 2 > area.bottom() {
            break;
        }

        let is_selected = idx == selected_index;

        // Create layout for this item (2 rows)
        let item_area = Rect {
            x: area.left() + 1,
            y: current_y,
            width: area.width.saturating_sub(2),
            height: 2,
        };

        render_download_item(f, item_area, download, is_selected);

        current_y += 2;
    }

    // Render border
    let title = format!(
        "━━ Downloads [{}/{}] ━━",
        selected_index + 1,
        downloads.len()
    );
    let border = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .border_style(Styles::border());

    f.render_widget(border, area);
}

/// Render a single download item
fn render_download_item(f: &mut Frame, area: Rect, download: &Download, is_selected: bool) {
    let item_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    // Determine if download has error
    let has_error = download.status == "ERROR" || download.status.contains("error");

    // Truncate name if too long
    let max_name_len = area.width.saturating_sub(10) as usize;
    let display_name = truncate_text(&download.name, max_name_len);

    // Add status indicator icon
    let status_icon = match download.status.as_str() {
        "ACTIVE" => "▶ ",
        "PAUSED" => "⏸ ",
        "WAITING" => "⏳ ",
        "COMPLETE" => "✓ ",
        "ERROR" => "✗ ",
        _ => "  ",
    };

    // Style based on selection and status
    let name_style = if has_error {
        Styles::error()
    } else if is_selected {
        Styles::selected().add_modifier(Modifier::BOLD)
    } else if download.progress >= 1.0 {
        Styles::success()
    } else {
        Styles::text()
    };

    // Render name with status icon
    let name_line = Line::from(vec![
        Span::styled(status_icon, name_style),
        Span::styled(display_name, name_style),
    ]);
    let name_paragraph = Paragraph::new(name_line);
    f.render_widget(name_paragraph, item_layout[0]);

    // Render progress gauge
    let gauge_label = if has_error {
        format!("ERROR: {}", download.status)
    } else {
        format!("{:.0}% • {}", download.progress * 100.0, download.speed)
    };

    let gauge_style = if has_error {
        Styles::error()
    } else if is_selected {
        Styles::selected()
    } else if download.progress >= 1.0 {
        Styles::success()
    } else {
        Styles::progress(&download.status, download.progress >= 1.0)
    };

    let gauge = LineGauge::default()
        .ratio(if has_error { 0.0 } else { download.progress })
        .label(gauge_label)
        .gauge_style(gauge_style);

    f.render_widget(gauge, item_layout[1]);
}

/// Validate and fix list selection
fn validate_selection(list_state: &mut ListState, item_count: usize) {
    if item_count == 0 {
        list_state.select(None);
    } else if list_state.selected().unwrap_or(0) >= item_count {
        list_state.select(Some(item_count.saturating_sub(1)));
    } else if list_state.selected().is_none() {
        list_state.select(Some(0));
    }
}

/// Calculate scroll offset to keep selected item visible
fn calculate_scroll_offset(selected_index: usize, items_per_screen: usize) -> usize {
    if items_per_screen == 0 {
        return 0;
    }

    if selected_index < items_per_screen {
        0
    } else {
        selected_index - items_per_screen + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_selection_empty() {
        let mut state = ListState::default();
        state.select(Some(5));
        validate_selection(&mut state, 0);
        assert_eq!(state.selected(), None);
    }

    #[test]
    fn test_validate_selection_out_of_bounds() {
        let mut state = ListState::default();
        state.select(Some(10));
        validate_selection(&mut state, 5);
        assert_eq!(state.selected(), Some(4));
    }

    #[test]
    fn test_calculate_scroll_offset() {
        assert_eq!(calculate_scroll_offset(0, 10), 0);
        assert_eq!(calculate_scroll_offset(5, 10), 0);
        assert_eq!(calculate_scroll_offset(15, 10), 6);
        assert_eq!(calculate_scroll_offset(20, 10), 11);
    }

    #[test]
    fn test_calculate_scroll_offset_zero_screen() {
        assert_eq!(calculate_scroll_offset(10, 0), 0);
    }
}
