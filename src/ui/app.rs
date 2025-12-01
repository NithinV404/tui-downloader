//! Main application rendering module
//!
//! This module orchestrates all UI components and handles the main rendering logic.

use crate::models::{Download, DownloadType, InputMode};
use crate::ui::utils::{count_by_tab, filter_by_tab};
use crate::ui::widgets::{
    format_tab_title, render_details_panel, render_downloads_list, render_input_field,
    render_shortcuts, render_status_bar, render_tabs,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::ListState,
};

/// Main application state for rendering
pub struct AppState<'a> {
    pub downloads: &'a [Download],
    pub current_tab: usize,
    pub input_text: &'a str,
    pub input_mode: InputMode,
    pub status_message: &'a str,
}

/// Render the complete application UI
///
/// # Arguments
/// * `f` - Frame to render to
/// * `state` - Application state
/// * `list_state` - Mutable list state for selection tracking
pub fn render(f: &mut Frame, state: AppState, list_state: &mut ListState) {
    let size = f.size();

    // Main vertical layout
    let main_layout = if !state.status_message.is_empty() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Status bar
            ])
            .split(size)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(size)
    };

    // Content layout: Input, Tabs, Main Area, Shortcuts
    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input field
            Constraint::Length(3), // Tabs
            Constraint::Min(1),    // Main area (downloads list + details)
            Constraint::Length(4), // Shortcuts guide (2 lines)
        ])
        .split(main_layout[0]);

    // Main area split: Downloads list | Details panel
    let main_area_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Downloads list
            Constraint::Percentage(60), // Details panel
        ])
        .split(content_layout[2]);

    // Render input field
    render_input_field(f, content_layout[0], state.input_text, state.input_mode);

    // Render tabs with counts
    render_tabs_with_counts(f, content_layout[1], state.downloads, state.current_tab);

    // Render downloads list
    let filtered_downloads = filter_by_tab(state.downloads, state.current_tab);
    render_downloads_list(f, main_area_layout[0], &filtered_downloads, list_state);

    // Render details panel
    let selected_download = get_selected_download(state.downloads, &filtered_downloads, list_state);
    render_details_panel(f, main_area_layout[1], &selected_download);

    // Render shortcuts guide
    render_shortcuts(f, content_layout[3], state.input_mode);

    // Render status bar if there's a message
    if !state.status_message.is_empty() {
        render_status_bar(f, main_layout[1], state.status_message);
    }
}

/// Render tabs with download counts
fn render_tabs_with_counts(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    downloads: &[Download],
    current_tab: usize,
) {
    let active_count = count_by_tab(downloads, 0);
    let queue_count = count_by_tab(downloads, 1);
    let completed_count = count_by_tab(downloads, 2);

    let tab1 = format_tab_title("Active", 1, active_count);
    let tab2 = format_tab_title("Queue", 2, queue_count);
    let tab3 = format_tab_title("Completed", 3, completed_count);

    let tab_titles = vec![tab1.as_str(), tab2.as_str(), tab3.as_str()];
    render_tabs(f, area, current_tab, tab_titles);
}

/// Get the currently selected download or a placeholder
fn get_selected_download<'a>(
    all_downloads: &'a [Download],
    filtered_downloads: &[&'a Download],
    list_state: &ListState,
) -> Download {
    let selected_index = list_state.selected().unwrap_or(0);

    if !filtered_downloads.is_empty() && selected_index < filtered_downloads.len() {
        (*filtered_downloads[selected_index]).clone()
    } else if !all_downloads.is_empty() {
        all_downloads[0].clone()
    } else {
        create_placeholder_download()
    }
}

/// Create a placeholder download when no downloads exist
fn create_placeholder_download() -> Download {
    Download {
        gid: None,
        name: "No downloads".to_string(),
        url: None,
        progress: 0.0,
        speed: "N/A".to_string(),
        status: "IDLE".to_string(),
        total_length: 0,
        completed_length: 0,
        download_type: DownloadType::Http,
        speed_history: Vec::new(),
        upload_speed: "N/A".to_string(),
        upload_speed_history: Vec::new(),
        connections: 0,
        file_path: None,
        error_message: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_download(name: &str, status: &str, progress: f64) -> Download {
        Download {
            gid: Some(format!("gid_{}", name)),
            name: name.to_string(),
            url: None,
            progress,
            speed: "0 B/s".to_string(),
            status: status.to_string(),
            total_length: 1024,
            completed_length: (1024.0 * progress) as u64,
            download_type: DownloadType::Http,
            speed_history: vec![],
            upload_speed: "0 B/s".to_string(),
            upload_speed_history: vec![],
            connections: 0,
            file_path: None,
            error_message: None,
        }
    }

    #[test]
    fn test_placeholder_download() {
        let placeholder = create_placeholder_download();
        assert_eq!(placeholder.name, "No downloads");
        assert_eq!(placeholder.status, "IDLE");
        assert_eq!(placeholder.progress, 0.0);
    }

    #[test]
    fn test_get_selected_download_empty() {
        let downloads: Vec<Download> = vec![];
        let filtered: Vec<&Download> = vec![];
        let mut state = ListState::default();
        state.select(Some(0));

        let result = get_selected_download(&downloads, &filtered, &state);
        assert_eq!(result.name, "No downloads");
    }

    #[test]
    fn test_get_selected_download_with_selection() {
        let downloads = vec![
            create_test_download("file1.txt", "ACTIVE", 0.5),
            create_test_download("file2.txt", "WAITING", 0.0),
        ];
        let filtered: Vec<&Download> = downloads.iter().collect();
        let mut state = ListState::default();
        state.select(Some(1));

        let result = get_selected_download(&downloads, &filtered, &state);
        assert_eq!(result.name, "file2.txt");
    }

    #[test]
    fn test_get_selected_download_out_of_bounds() {
        let downloads = vec![create_test_download("file1.txt", "ACTIVE", 0.5)];
        let filtered: Vec<&Download> = vec![];
        let mut state = ListState::default();
        state.select(Some(10));

        let result = get_selected_download(&downloads, &filtered, &state);
        // Should fall back to first download from all_downloads
        assert_eq!(result.name, "file1.txt");
    }
}
