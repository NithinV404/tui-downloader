//! UI module for the TUI downloader
//!
//! This module provides a clean, modular UI architecture with reusable components.
//!
//! # Architecture
//!
//! - `app.rs` - Main application rendering orchestration
//! - `widgets/` - Reusable UI components
//! - `theme.rs` - Consistent color scheme and styles
//! - `utils.rs` - Helper functions for filtering and formatting
//!
//! # Usage
//!
//! ```no_run
//! use tui_downloader::ui::{render_app, AppState};
//! use tui_downloader::models::InputMode;
//! use ratatui::widgets::ListState;
//!
//! // In your main rendering loop:
//! let state = AppState {
//!     downloads: &all_downloads,
//!     current_tab: 0,
//!     input_text: "https://example.com/file.zip",
//!     input_mode: InputMode::Normal,
//!     status_message: "",
//! };
//!
//! render_app(frame, state, &mut list_state);
//! ```

mod app;
mod theme;
mod utils;
pub mod widgets;

// Re-export main rendering function and types
pub use app::AppState;

// Re-export utilities used by main
pub use utils::filter_by_tab;

// Re-export popup functions
pub use widgets::{PopupType, render_popup, render_size_warning};

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
/// * `input_mode` - Current input mode (Normal/Editing)
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
    };

    app::render(f, state, list_state);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state() {
        // Test that AppState is exported
        use crate::models::InputMode;
        let _state = AppState {
            downloads: &[],
            current_tab: 0,
            input_text: "",
            input_mode: InputMode::Normal,
            status_message: "",
        };
    }
}
