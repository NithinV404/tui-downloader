//! UI module for the TUI downloader
//!
//! This module provides a clean, modular UI architecture with reusable components.
//!
//! # Architecture
//!
//! - `app.rs` - Main application rendering orchestration
//! - `widgets/` - Reusable UI components
//! - `theme.rs` - Consistent color scheme and styles
//! - `utils.rs` - Helper functions for filtering, sorting, and formatting
//!
//! # Features
//!
//! - Global statistics bar showing aggregate download info
//! - Search/filter functionality
//! - Sorting by name, size, progress, speed, status
//! - Help popup with all keybindings
//! - Speed limit configuration popup
//! - ETA (estimated time of arrival) display
//! - Enhanced torrent info (seeds/peers)
//!
//! # Usage
//!
//! ```ignore
//! use tui_downloader::ui::{render_app_full, AppState};
//! use tui_downloader::models::InputMode;
//! use ratatui::widgets::ListState;
//!
//! // In your main rendering loop:
//! let downloads = vec![];
//! let state = AppState {
//!     downloads: &downloads,
//!     current_tab: 0,
//!     input_text: "https://example.com/file.zip",
//!     input_mode: InputMode::Normal,
//!     status_message: "",
//!     search_query: "",
//!     ..Default::default()
//! };
//!
//! let mut list_state = ListState::default();
//! render_app_full(frame, state, &mut list_state);
//! ```

mod app;
pub mod theme;
pub mod utils;
pub mod widgets;

// Re-export main rendering function and types
pub use app::AppState;

// Re-export utilities used by main
pub use utils::{filter_by_tab, format_speed, SortOrder};

// Re-export popup functions and types
pub use widgets::{render_popup, render_size_warning, PopupType, SpeedLimitState};

/// Main render function for the application
///
/// This is a convenience wrapper around `app::render` that provides
/// a simpler API for the most common use case.
///
/// # Arguments
/// * `f` - Frame to render to
/// * `downloads` - Slice of all downloads
/// * `current_tab` - Index of currently selected tab (0=Active, 1=Queue, 2=Completed)
/// * `list_state` - Mutable list state for selection tracking
/// * `input_text` - Current input text
/// * `input_mode` - Current input mode (Normal/Editing/Search/etc.)
/// * `status_message` - Optional status message to display
///
/// # Example
///
/// ```no_run
/// use ratatui::Frame;
/// use ratatui::widgets::ListState;
/// use tui_downloader::models::{Download, InputMode};
///
/// fn render_ui(
///     f: &mut Frame,
///     downloads: &[Download],
///     tab: usize,
///     list_state: &mut ListState,
///     input: &str,
///     mode: InputMode,
/// ) {
///     tui_downloader::ui::render_app(
///         f,
///         downloads,
///         tab,
///         list_state,
///         input,
///         mode,
///         "",
///     );
/// }
/// ```
#[allow(dead_code)]
pub fn render_app(
    f: &mut ratatui::Frame,
    downloads: &[crate::models::Download],
    current_tab: usize,
    list_state: &mut ratatui::widgets::ListState,
    input_text: &str,
    input_mode: crate::models::InputMode,
    status_message: &str,
) {
    let state = AppState {
        downloads,
        current_tab,
        input_text,
        input_mode,
        status_message,
        ..Default::default()
    };

    app::render(f, state, list_state);
}

/// Extended render function with all new features
///
/// This function provides full control over all UI features including
/// search, sorting, speed limits, and popups.
///
/// # Arguments
/// * `f` - Frame to render to
/// * `state` - Complete application state
/// * `list_state` - Mutable list state for selection tracking
pub fn render_app_full(
    f: &mut ratatui::Frame,
    state: AppState,
    list_state: &mut ratatui::widgets::ListState,
) {
    app::render(f, state, list_state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::InputMode;

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert_eq!(state.current_tab, 0);
        assert!(state.search_query.is_empty());
        assert_eq!(state.sort_order, SortOrder::Name);
        assert!(state.sort_ascending);
    }

    #[test]
    fn test_app_state_with_values() {
        let downloads: Vec<crate::models::Download> = vec![];
        let state = AppState {
            downloads: &downloads,
            current_tab: 1,
            input_text: "test",
            input_mode: InputMode::Editing,
            status_message: "Hello",
            search_query: "query",
            sort_order: SortOrder::Size,
            sort_ascending: false,
            help_scroll: 5,
            speed_limit_state: None,
            download_limit: 1024,
            upload_limit: 512,
            selected_indices: &[0, 1, 2],
        };

        assert_eq!(state.current_tab, 1);
        assert_eq!(state.input_text, "test");
        assert_eq!(state.search_query, "query");
        assert_eq!(state.sort_order, SortOrder::Size);
        assert!(!state.sort_ascending);
        assert_eq!(state.help_scroll, 5);
        assert_eq!(state.download_limit, 1024);
        assert_eq!(state.upload_limit, 512);
        assert_eq!(state.selected_indices.len(), 3);
    }

    #[test]
    fn test_exports() {
        // Test that all re-exports are accessible
        let _ = SortOrder::Name;
        let _ = PopupType::Info;
    }
}
