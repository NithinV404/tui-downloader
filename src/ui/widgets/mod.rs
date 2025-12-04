//! UI widgets module
//!
//! This module contains all reusable UI components for the TUI downloader.
//! Each widget is self-contained and can be used independently.

pub mod details_panel;
pub mod downloads_list;
pub mod global_stats;
pub mod help_popup;
pub mod input_field;
pub mod popup;
pub mod search_bar;
pub mod shortcuts;
pub mod speed_limit_popup;
pub mod status_bar;
pub mod tabs;

// Re-export widget render functions for convenience
pub use details_panel::render as render_details_panel;
pub use help_popup::render as render_help_popup;
pub use input_field::render as render_input_field;
pub use popup::{render as render_popup, render_size_warning, PopupType};
pub use search_bar::render as render_search_bar;
pub use speed_limit_popup::render as render_speed_limit_popup;
pub use speed_limit_popup::SpeedLimitState;
pub use status_bar::render as render_status_bar;
