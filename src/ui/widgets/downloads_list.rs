//! Downloads list widget for displaying download items

use crate::models::Download;
use crate::ui::theme::{Styles, Theme};
use crate::ui::utils::{format_download_eta, truncate_text};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, ListState, Paragraph},
    Frame,
};

/// Render the downloads list widget
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Area to render in
/// * `downloads` - Slice of downloads to display
/// * `list_state` - Mutable list state for selection tracking
#[allow(dead_code)]
pub fn render(f: &mut Frame, area: Rect, downloads: &[&Download], list_state: &mut ListState) {
    render_with_search(f, area, downloads, list_state, "", &[])
}

/// Render the downloads list widget with search highlighting
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Area to render in
/// * `downloads` - Slice of downloads to display
/// * `list_state` - Mutable list state for selection tracking
/// * `search_query` - Current search query for highlighting
/// * `selected_indices` - Indices of selected items for batch operations
pub fn render_with_search(
    f: &mut Frame,
    area: Rect,
    downloads: &[&Download],
    list_state: &mut ListState,
    search_query: &str,
    selected_indices: &[usize],
) {
    // Validate and adjust list state
    validate_selection(list_state, downloads.len());

    let selected_index = list_state.selected().unwrap_or(0);

    // Calculate scrolling - each item takes 3 rows (2 for content + 1 for separator)
    let visible_rows = area.height.saturating_sub(2) as usize; // -2 for borders
    let items_per_screen = visible_rows / 3;
    let scroll_offset = calculate_scroll_offset(selected_index, items_per_screen);

    // Build title with count
    let title = if downloads.is_empty() {
        " Downloads ".to_string()
    } else {
        format!(
            " Downloads [{}/{}] ",
            if downloads.is_empty() {
                0
            } else {
                selected_index + 1
            },
            downloads.len()
        )
    };

    // Render container with rounded border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(title)
        .border_style(Styles::border());

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Empty state
    if downloads.is_empty() {
        let empty_msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "No downloads in this category",
                Styles::text_muted(),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Styles::text_muted()),
                Span::styled("i", Styles::highlight()),
                Span::styled(" to add a download", Styles::text_muted()),
            ]),
        ]);
        f.render_widget(empty_msg, inner);
        return;
    }

    // Render items
    let mut current_y = inner.top();

    for (idx, download) in downloads.iter().enumerate() {
        // Skip items before scroll offset
        if idx < scroll_offset {
            continue;
        }

        // Stop if we've filled the visible area (need 3 rows: 2 for item + 1 for separator)
        if current_y + 3 > inner.bottom() && idx < downloads.len() - 1 {
            break;
        }
        if current_y + 2 > inner.bottom() {
            break;
        }

        let is_selected = idx == selected_index;
        let is_batch_selected = selected_indices.contains(&idx);

        // Create layout for this item (2 rows)
        let item_area = Rect {
            x: inner.left(),
            y: current_y,
            width: inner.width,
            height: 2,
        };

        render_download_item(
            f,
            item_area,
            download,
            is_selected,
            is_batch_selected,
            search_query,
        );

        current_y += 2;

        // Add separator line between items (not after the last one)
        if idx < downloads.len() - 1 && current_y < inner.bottom() {
            let separator_area = Rect {
                x: inner.left() + 1,
                y: current_y,
                width: inner.width.saturating_sub(2),
                height: 1,
            };
            let separator = Line::from(vec![Span::styled(
                "─".repeat(separator_area.width as usize),
                Style::default().fg(Theme::BORDER),
            )]);
            f.render_widget(Paragraph::new(separator), separator_area);
            current_y += 1;
        }
    }
}

/// Render a single download item
fn render_download_item(
    f: &mut Frame,
    area: Rect,
    download: &Download,
    is_selected: bool,
    is_batch_selected: bool,
    search_query: &str,
) {
    let item_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    // Determine if download has error
    let has_error = download.status == "ERROR" || download.status.to_lowercase().contains("error");

    // Status indicator icons
    let status_icon = match download.status.as_str() {
        "ACTIVE" => ">",
        "PAUSED" => "||",
        "WAITING" => "o",
        "COMPLETE" => "*",
        "ERROR" => "x",
        _ => "-",
    };

    // Selection marker
    let selection_marker = if is_selected {
        ">> "
    } else if is_batch_selected {
        " * "
    } else {
        "   "
    };

    // Style based on selection and status
    let name_style = if has_error {
        Styles::error()
    } else if is_selected {
        Style::default()
            .fg(Theme::HIGHLIGHT)
            .add_modifier(Modifier::BOLD)
    } else if is_batch_selected {
        Style::default()
            .fg(Theme::SELECTED)
            .add_modifier(Modifier::BOLD)
    } else if download.progress >= 1.0 {
        Style::default().fg(Theme::STATUS_COMPLETE)
    } else {
        Style::default().fg(Theme::CMD_COLOR)
    };

    let icon_style = if has_error {
        Styles::error()
    } else {
        match download.status.as_str() {
            "ACTIVE" => Style::default().fg(Theme::SUCCESS),
            "PAUSED" => Style::default().fg(Theme::WARNING),
            "WAITING" => Style::default().fg(Theme::TEXT_MUTED),
            "COMPLETE" => Style::default().fg(Theme::STATUS_COMPLETE),
            _ => Style::default().fg(Theme::TEXT_MUTED),
        }
    };

    // Truncate name if needed - leave room for status and ETA
    let max_name_len = area.width.saturating_sub(20) as usize;
    let display_name = truncate_text(&download.name, max_name_len);

    // Calculate ETA
    let eta = format_download_eta(download);

    // Build name line with search highlighting
    let name_spans = if !search_query.is_empty() {
        highlight_search(&display_name, search_query, name_style, is_selected)
    } else {
        vec![Span::styled(display_name, name_style)]
    };

    // Build first line: marker + icon + name + ETA
    let marker_style = if is_selected {
        Style::default()
            .fg(Theme::HIGHLIGHT)
            .add_modifier(Modifier::BOLD)
    } else if is_batch_selected {
        Style::default().fg(Theme::SELECTED)
    } else {
        Style::default().fg(Theme::TEXT_MUTED)
    };

    let mut name_line_spans = vec![
        Span::styled(selection_marker, marker_style),
        Span::styled(format!("{} ", status_icon), icon_style),
    ];
    name_line_spans.extend(name_spans);

    // Add ETA on the right side for active downloads
    if download.progress < 1.0 && download.status != "COMPLETE" && !eta.is_empty() {
        name_line_spans.push(Span::styled(
            format!("  {}", eta),
            Style::default().fg(Theme::TEXT_MUTED),
        ));
    }

    let name_line = Line::from(name_line_spans);
    let name_paragraph = Paragraph::new(name_line);
    f.render_widget(name_paragraph, item_layout[0]);

    // Second line: progress bar with info
    let progress_area = Rect {
        x: item_layout[1].x + 6, // Indent to align with name (after marker + icon)
        y: item_layout[1].y,
        width: item_layout[1].width.saturating_sub(6),
        height: 1,
    };

    if has_error {
        // Show error message instead of progress
        let error_msg = download
            .error_message
            .as_deref()
            .unwrap_or("Download failed");
        let error_line = Line::from(vec![Span::styled(
            truncate_text(error_msg, progress_area.width as usize),
            Styles::error(),
        )]);
        f.render_widget(Paragraph::new(error_line), progress_area);
    } else {
        // Progress bar with inline stats
        let progress_label = build_progress_label(download);

        let gauge_style = if download.progress >= 1.0 {
            Style::default()
                .fg(Theme::STATUS_COMPLETE)
                .bg(Theme::BACKGROUND)
        } else if download.status == "PAUSED" {
            Style::default()
                .fg(Theme::STATUS_PAUSED)
                .bg(Theme::BACKGROUND)
        } else {
            Style::default().fg(Theme::SUCCESS).bg(Theme::BACKGROUND)
        };

        let gauge = Gauge::default()
            .ratio(download.progress)
            .label(progress_label)
            .gauge_style(gauge_style);

        f.render_widget(gauge, progress_area);
    }
}

/// Build a clean progress label
fn build_progress_label(download: &Download) -> String {
    let percent = format!("{:.0}%", download.progress * 100.0);
    let size = crate::ui::utils::format_size(download.completed_length);

    // For torrents, include seeds/peers
    if download.download_type == crate::models::DownloadType::Torrent {
        format!(
            "{} | {} | {} | S:{} P:{}",
            percent, download.speed, size, download.seeds, download.peers
        )
    } else {
        format!("{} | {} | {}", percent, download.speed, size)
    }
}

/// Highlight search matches in text
fn highlight_search(
    text: &str,
    query: &str,
    base_style: ratatui::style::Style,
    _is_selected: bool,
) -> Vec<Span<'static>> {
    if query.is_empty() {
        return vec![Span::styled(text.to_string(), base_style)];
    }

    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    let mut spans = Vec::new();
    let mut last_end = 0;

    for (start, _) in text_lower.match_indices(&query_lower) {
        // Add text before match
        if start > last_end {
            spans.push(Span::styled(text[last_end..start].to_string(), base_style));
        }

        // Add highlighted match (preserve original case)
        let match_text = &text[start..start + query.len()];
        spans.push(Span::styled(
            match_text.to_string(),
            Style::default()
                .fg(Theme::HIGHLIGHT)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ));

        last_end = start + query.len();
    }

    // Add remaining text
    if last_end < text.len() {
        spans.push(Span::styled(text[last_end..].to_string(), base_style));
    }

    if spans.is_empty() {
        spans.push(Span::styled(text.to_string(), base_style));
    }

    spans
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
    use crate::models::DownloadType;

    fn create_test_download(name: &str, status: &str, progress: f64) -> Download {
        Download {
            gid: Some("test".to_string()),
            name: name.to_string(),
            url: Some("https://example.com/file.zip".to_string()),
            progress,
            speed: "1.5 MB/s".to_string(),
            status: status.to_string(),
            total_length: 1024 * 1024 * 100,
            completed_length: (1024 * 1024 * 100) as u64 * progress as u64,
            download_type: DownloadType::Http,
            speed_history: vec![1024 * 1024],
            upload_speed: "0 B/s".to_string(),
            upload_speed_history: vec![0],
            connections: 4,
            file_path: None,
            error_message: None,
            added_at: std::time::Instant::now(),
            seeds: 0,
            peers: 0,
            bitfield: None,
            num_pieces: 0,
        }
    }

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
    fn test_validate_selection_none() {
        let mut state = ListState::default();
        validate_selection(&mut state, 5);
        assert_eq!(state.selected(), Some(0));
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

    #[test]
    fn test_highlight_search_no_match() {
        let spans = highlight_search("test file.zip", "xyz", Styles::text(), false);
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn test_highlight_search_single_match() {
        let spans = highlight_search("test file.zip", "file", Styles::text(), false);
        assert_eq!(spans.len(), 3); // "test ", "file", ".zip"
    }

    #[test]
    fn test_highlight_search_multiple_matches() {
        let spans = highlight_search("file test file", "file", Styles::text(), false);
        assert_eq!(spans.len(), 3); // "file", " test ", "file" (no trailing empty)
    }

    #[test]
    fn test_highlight_search_case_insensitive() {
        let spans = highlight_search("TEST File.zip", "test", Styles::text(), false);
        assert!(spans.len() >= 2); // Should find match despite case difference
    }

    #[test]
    fn test_highlight_search_empty_query() {
        let spans = highlight_search("test file.zip", "", Styles::text(), false);
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn test_download_item_status_icons() {
        let download = create_test_download("test.zip", "ACTIVE", 0.5);
        assert_eq!(download.status, "ACTIVE");

        let download = create_test_download("test.zip", "PAUSED", 0.5);
        assert_eq!(download.status, "PAUSED");

        let download = create_test_download("test.zip", "COMPLETE", 1.0);
        assert_eq!(download.status, "COMPLETE");
    }

    #[test]
    fn test_build_progress_label() {
        let download = create_test_download("test.zip", "ACTIVE", 0.5);
        let label = build_progress_label(&download);
        assert!(label.contains("50%"));
        assert!(label.contains("1.5 MB/s"));
    }
}
