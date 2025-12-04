//! Main application rendering module
//!
//! This module orchestrates all UI components and handles the main rendering logic.

use crate::models::{Download, DownloadType, InputMode};
use crate::ui::theme::{Styles, Theme};
use crate::ui::utils::{
    calculate_global_stats, count_by_tab, filter_by_search, filter_by_tab, format_speed,
    sort_downloads, GlobalStats, SortOrder,
};
use crate::ui::widgets::downloads_list::render_with_search;
use crate::ui::widgets::{
    render_details_panel, render_help_popup, render_input_field, render_search_bar,
    render_speed_limit_popup, render_status_bar, SpeedLimitState,
};
use ratatui::symbols::border;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListState as TabListState, Paragraph};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    Frame,
};

/// Application version for title banner
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Main application state for rendering
pub struct AppState<'a> {
    pub downloads: &'a [Download],
    pub current_tab: usize,
    pub input_text: &'a str,
    pub input_mode: InputMode,
    pub status_message: &'a str,
    // New fields for enhanced features
    pub search_query: &'a str,
    pub sort_order: SortOrder,
    pub sort_ascending: bool,
    pub help_scroll: usize,
    pub speed_limit_state: Option<&'a SpeedLimitState>,
    #[allow(dead_code)]
    pub download_limit: u64,
    #[allow(dead_code)]
    pub upload_limit: u64,
    pub selected_indices: &'a [usize],
}

impl<'a> Default for AppState<'a> {
    fn default() -> Self {
        Self {
            downloads: &[],
            current_tab: 0,
            input_text: "",
            input_mode: InputMode::Normal,
            status_message: "",
            search_query: "",
            sort_order: SortOrder::Name,
            sort_ascending: true,
            help_scroll: 0,
            speed_limit_state: None,
            download_limit: 0,
            upload_limit: 0,
            selected_indices: &[],
        }
    }
}

/// Render the complete application UI
pub fn render(f: &mut Frame, state: AppState, list_state: &mut ratatui::widgets::ListState) {
    let size = f.size();

    // Calculate global stats
    let global_stats = calculate_global_stats(state.downloads);

    // Main vertical layout: content area + shortcuts + optional status
    let has_status = !state.status_message.is_empty();
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if has_status {
            vec![
                Constraint::Min(10),   // Main content area
                Constraint::Length(3), // Shortcuts bar
                Constraint::Length(1), // Status message
            ]
        } else {
            vec![
                Constraint::Min(10),   // Main content area
                Constraint::Length(3), // Shortcuts bar
            ]
        })
        .split(size);

    // Horizontal split: left sidebar | main content
    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22), // Left sidebar
            Constraint::Min(40),    // Main content area
        ])
        .split(main_layout[0]);

    // Left sidebar: title banner + category tabs
    let left_sidebar = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Title banner
            Constraint::Min(5),    // Category tabs
        ])
        .split(horizontal_layout[0]);

    // Render title banner
    render_title_banner(f, left_sidebar[0]);

    // Render category tabs in left sidebar
    let mut tab_state = TabListState::default();
    tab_state.select(Some(state.current_tab));
    render_category_tabs(
        f,
        left_sidebar[1],
        state.downloads,
        state.current_tab,
        &mut tab_state,
    );

    // Right content area: input field + downloads/details
    let right_content = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input field
            Constraint::Min(5),    // Downloads + Details
        ])
        .split(horizontal_layout[1]);

    // Render input field
    render_input_field(f, right_content[0], state.input_text, state.input_mode);

    // Downloads and details split
    let content_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(45), // Downloads list
            Constraint::Percentage(55), // Details panel
        ])
        .split(right_content[1]);

    // Filter and sort downloads
    let filtered_by_tab = filter_by_tab(state.downloads, state.current_tab);
    let filtered_downloads = if state.search_query.is_empty() {
        filtered_by_tab
    } else {
        filter_by_search(&filtered_by_tab, state.search_query)
    };

    // Sort downloads
    let mut sorted_downloads = filtered_downloads.clone();
    sort_downloads(
        &mut sorted_downloads,
        state.sort_order,
        state.sort_ascending,
    );

    // Render downloads list with search highlighting
    render_with_search(
        f,
        content_split[0],
        &sorted_downloads,
        list_state,
        state.search_query,
        state.selected_indices,
    );

    // Render details panel
    let selected_download = get_selected_download(state.downloads, &sorted_downloads, list_state);
    render_details_panel(f, content_split[1], &selected_download);

    // Render keyboard shortcuts bar
    render_shortcuts_bar(f, main_layout[1], state.input_mode, &global_stats);

    // Render status bar if there's a message
    if has_status {
        render_status_bar(f, main_layout[2], state.status_message);
    }

    // Render overlays/popups last so they appear on top

    // Search bar overlay
    if state.input_mode == InputMode::Search {
        let result_count = sorted_downloads.len();
        let total_count = filter_by_tab(state.downloads, state.current_tab).len();
        render_search_bar(f, size, state.search_query, result_count, total_count);
    }

    // Help popup
    if state.input_mode == InputMode::Help {
        render_help_popup(f, size, state.help_scroll);
    }

    // Speed limit popup
    if state.input_mode == InputMode::SpeedLimit {
        if let Some(speed_state) = state.speed_limit_state {
            render_speed_limit_popup(f, size, speed_state);
        }
    }
}

/// Render the title banner with decorative borders
fn render_title_banner(f: &mut Frame, area: Rect) {
    // Create decorative border style
    let label_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::Set {
            top_left: " ",
            top_right: " ",
            bottom_left: " ",
            bottom_right: " ",
            vertical_left: " ",
            vertical_right: " ",
            horizontal_top: "*",
            horizontal_bottom: "*",
        })
        .border_style(Style::default().fg(Theme::SECONDARY));

    let title_text = vec![
        Line::from(vec![
            Span::styled(
                "TUI",
                Style::default()
                    .fg(Theme::HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Downloader",
                Style::default()
                    .fg(Theme::TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![Span::styled(
            format!("v{}", APP_VERSION),
            Style::default()
                .fg(Theme::TEXT_MUTED)
                .add_modifier(Modifier::ITALIC),
        )]),
    ];

    let label = Paragraph::new(title_text)
        .block(label_block)
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(label, area);
}

/// Render category tabs in the left sidebar
fn render_category_tabs(
    f: &mut Frame,
    area: Rect,
    downloads: &[Download],
    current_tab: usize,
    tab_state: &mut TabListState,
) {
    let active_count = count_by_tab(downloads, 0);
    let queue_count = count_by_tab(downloads, 1);
    let completed_count = count_by_tab(downloads, 2);

    let tabs = vec![
        format_tab_item("Active", active_count, 0, current_tab),
        format_tab_item("Queue", queue_count, 1, current_tab),
        format_tab_item("Completed", completed_count, 2, current_tab),
    ];

    let tab_list = List::new(tabs)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .title(" Categories ")
                .border_style(Styles::border()),
        )
        .highlight_style(
            Style::default()
                .fg(Theme::HIGHLIGHT)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(tab_list, area, tab_state);
}

/// Format a tab item with count
fn format_tab_item(name: &str, count: usize, index: usize, current: usize) -> Line<'static> {
    let style = if index == current {
        Style::default()
            .fg(Theme::HIGHLIGHT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::TEXT_MUTED)
    };

    let icon = match index {
        0 => ">", // Active - playing
        1 => "o", // Queue - waiting
        2 => "*", // Completed - done
        _ => "-",
    };

    Line::from(vec![
        Span::styled(format!("{} ", icon), style),
        Span::styled(format!("{}", name), style),
        Span::styled(
            format!(" ({})", count),
            Style::default().fg(Theme::TEXT_MUTED),
        ),
    ])
}

/// Render keyboard shortcuts bar at the bottom
fn render_shortcuts_bar(f: &mut Frame, area: Rect, mode: InputMode, stats: &GlobalStats) {
    let (scope_name, shortcuts) = get_shortcuts_for_mode(mode);

    // Build shortcut spans
    let mut spans: Vec<Span> = vec![Span::styled(" ", Styles::text_muted())];

    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        spans.push(Span::styled(
            format!("{}", key),
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(format!(" {} ", desc), Styles::text_muted()));

        if i < shortcuts.len() - 1 {
            spans.push(Span::styled(" ", Styles::text_muted()));
        }
    }

    // Add global speed info on the right
    let speed_info = format!(
        "  D: {} | U: {}",
        format_speed(stats.total_download_speed),
        format_speed(stats.total_upload_speed)
    );

    // Calculate padding
    let left_text: String = spans.iter().map(|s| s.content.to_string()).collect();
    let left_len = left_text.chars().count();
    let speed_len = speed_info.chars().count();
    let total_width = area.width.saturating_sub(4) as usize; // Account for borders
    let padding = total_width.saturating_sub(left_len + speed_len);

    spans.push(Span::styled(" ".repeat(padding), Styles::text_muted()));
    spans.push(Span::styled(speed_info, Style::default().fg(Theme::INFO)));

    let shortcuts_paragraph = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(format!(" {} ", scope_name))
            .border_style(Styles::border()),
    );

    f.render_widget(shortcuts_paragraph, area);
}

/// Get shortcuts based on input mode (returns scope name and key-description pairs)
fn get_shortcuts_for_mode(mode: InputMode) -> (&'static str, Vec<(&'static str, &'static str)>) {
    match mode {
        InputMode::Editing => (
            "Add Download",
            vec![("Enter", "submit"), ("Esc", "cancel"), ("Ctrl+U", "clear")],
        ),
        InputMode::Search => (
            "Search",
            vec![
                ("Enter", "apply"),
                ("Esc", "clear"),
                ("Backspace", "delete"),
            ],
        ),
        InputMode::SpeedLimit => (
            "Speed Limit",
            vec![
                ("Tab", "switch"),
                ("j/k", "adjust"),
                ("Enter", "apply"),
                ("Esc", "cancel"),
            ],
        ),
        InputMode::Help => (
            "Help",
            vec![("j/k", "scroll"), ("Esc", "close"), ("q", "close")],
        ),
        InputMode::Confirmation => (
            "Confirm",
            vec![("y", "yes"), ("n", "no"), ("Esc", "cancel")],
        ),
        InputMode::Settings => (
            "Settings",
            vec![("j/k", "navigate"), ("Enter", "edit"), ("Esc", "close")],
        ),
        InputMode::Normal => (
            "Downloads",
            vec![
                ("i", "add"),
                ("/", "search"),
                ("Space", "pause"),
                ("d", "delete"),
                ("1-3", "tabs"),
                ("?", "help"),
                ("q", "quit"),
            ],
        ),
    }
}

/// Get the currently selected download or a placeholder
fn get_selected_download<'a>(
    all_downloads: &'a [Download],
    filtered_downloads: &[&'a Download],
    list_state: &ratatui::widgets::ListState,
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
        added_at: std::time::Instant::now(),
        seeds: 0,
        peers: 0,
        bitfield: None,
        num_pieces: 0,
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
            added_at: std::time::Instant::now(),
            seeds: 0,
            peers: 0,
            bitfield: None,
            num_pieces: 0,
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
        let mut state = ratatui::widgets::ListState::default();
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
        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(1));

        let result = get_selected_download(&downloads, &filtered, &state);
        assert_eq!(result.name, "file2.txt");
    }

    #[test]
    fn test_get_selected_download_out_of_bounds() {
        let downloads = vec![create_test_download("file1.txt", "ACTIVE", 0.5)];
        let filtered: Vec<&Download> = vec![];
        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(10));

        let result = get_selected_download(&downloads, &filtered, &state);
        // Should fall back to first download from all_downloads
        assert_eq!(result.name, "file1.txt");
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert_eq!(state.current_tab, 0);
        assert!(state.search_query.is_empty());
        assert_eq!(state.sort_order, SortOrder::Name);
        assert!(state.sort_ascending);
        assert_eq!(state.download_limit, 0);
        assert_eq!(state.upload_limit, 0);
    }

    #[test]
    fn test_format_tab_item_active() {
        let line = format_tab_item("Active", 5, 0, 0);
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_format_tab_item_inactive() {
        let line = format_tab_item("Queue", 3, 1, 0);
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_shortcuts_for_all_modes() {
        let modes = vec![
            InputMode::Normal,
            InputMode::Editing,
            InputMode::Search,
            InputMode::SpeedLimit,
            InputMode::Help,
            InputMode::Confirmation,
            InputMode::Settings,
        ];

        for mode in modes {
            let (scope, shortcuts) = get_shortcuts_for_mode(mode);
            assert!(
                !scope.is_empty(),
                "Mode {:?} should have a scope name",
                mode
            );
            assert!(
                !shortcuts.is_empty(),
                "Mode {:?} should have shortcuts",
                mode
            );
        }
    }
}
