//! UI widgets module
//!
//! This module contains all reusable UI components for the TUI downloader.
//! Each widget is self-contained and can be used independently.

pub mod details_panel;
pub mod downloads_list;
pub mod input_field;
pub mod popup;
pub mod shortcuts;
pub mod status_bar;
pub mod tabs;

// Re-export widget render functions for convenience
pub use details_panel::render as render_details_panel;
pub use downloads_list::render as render_downloads_list;
pub use input_field::render as render_input_field;
pub use popup::{PopupType, render as render_popup, render_size_warning};
pub use shortcuts::render as render_shortcuts;
pub use status_bar::render as render_status_bar;
pub use tabs::{format_tab_title, render as render_tabs};
