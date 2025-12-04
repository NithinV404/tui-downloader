//! Theme and styling constants for the TUI

use ratatui::style::{Color, Modifier, Style};

/// Application color scheme
pub struct Theme;

impl Theme {
    // Primary colors
    #[allow(dead_code)]
    pub const PRIMARY: Color = Color::Rgb(139, 233, 253);
    pub const SECONDARY: Color = Color::Rgb(255, 255, 85);
    pub const SUCCESS: Color = Color::Rgb(5, 255, 55);
    pub const WARNING: Color = Color::Rgb(255, 199, 119);
    pub const ERROR: Color = Color::Rgb(199, 55, 44);
    pub const INFO: Color = Color::LightBlue;

    // UI element colors
    pub const BORDER: Color = Color::Gray;
    pub const BORDER_FOCUSED: Color = Color::LightBlue;
    pub const TEXT: Color = Color::Rgb(204, 224, 208);
    pub const TEXT_MUTED: Color = Color::DarkGray;
    pub const BACKGROUND: Color = Color::Black;

    // Download status colors
    pub const STATUS_ACTIVE: Color = Color::Rgb(5, 255, 55);
    pub const STATUS_PAUSED: Color = Color::Rgb(255, 255, 85);
    pub const STATUS_WAITING: Color = Color::LightBlue;
    pub const STATUS_COMPLETE: Color = Color::Blue;
    pub const STATUS_ERROR: Color = Color::Rgb(199, 55, 44);
    #[allow(dead_code)]
    pub const STATUS_IDLE: Color = Color::DarkGray;

    // Progress colors
    #[allow(dead_code)]
    pub const PROGRESS_ACTIVE: Color = Color::Rgb(5, 255, 55);
    #[allow(dead_code)]
    pub const PROGRESS_PAUSED: Color = Color::Rgb(255, 255, 85);
    #[allow(dead_code)]
    pub const PROGRESS_COMPLETE: Color = Color::Blue;

    // Highlight colors
    pub const HIGHLIGHT: Color = Color::Rgb(255, 255, 85);
    pub const SELECTED: Color = Color::LightBlue;

    // Category colors
    pub const CMD_COLOR: Color = Color::Rgb(204, 224, 208);
}

/// Common styles used throughout the application
pub struct Styles;

#[allow(dead_code)]
impl Styles {
    /// Default text style
    pub fn text() -> Style {
        Style::default().fg(Theme::TEXT)
    }

    /// Muted/secondary text style
    pub fn text_muted() -> Style {
        Style::default().fg(Theme::TEXT_MUTED)
    }

    /// Highlighted text style
    pub fn highlight() -> Style {
        Style::default()
            .fg(Theme::HIGHLIGHT)
            .add_modifier(Modifier::BOLD)
    }

    /// Selected item style
    pub fn selected() -> Style {
        Style::default()
            .fg(Theme::SELECTED)
            .add_modifier(Modifier::BOLD)
    }

    /// Error text style
    pub fn error() -> Style {
        Style::default()
            .fg(Theme::ERROR)
            .add_modifier(Modifier::BOLD)
    }

    /// Success text style
    pub fn success() -> Style {
        Style::default()
            .fg(Theme::SUCCESS)
            .add_modifier(Modifier::BOLD)
    }

    /// Warning text style
    pub fn warning() -> Style {
        Style::default()
            .fg(Theme::WARNING)
            .add_modifier(Modifier::BOLD)
    }

    /// Info text style
    pub fn info() -> Style {
        Style::default().fg(Theme::INFO)
    }

    /// Border style
    pub fn border() -> Style {
        Style::default().fg(Theme::BORDER)
    }

    /// Focused border style
    pub fn border_focused() -> Style {
        Style::default()
            .fg(Theme::BORDER_FOCUSED)
            .add_modifier(Modifier::BOLD)
    }

    /// Status-specific style
    pub fn status(status: &str) -> Style {
        let color = match status {
            "ACTIVE" => Theme::STATUS_ACTIVE,
            "PAUSED" => Theme::STATUS_PAUSED,
            "WAITING" => Theme::STATUS_WAITING,
            "COMPLETE" => Theme::STATUS_COMPLETE,
            "ERROR" => Theme::STATUS_ERROR,
            _ => Theme::STATUS_IDLE,
        };

        Style::default().fg(color).add_modifier(Modifier::BOLD)
    }

    /// Progress bar style based on status
    pub fn progress(status: &str, complete: bool) -> Style {
        let color = if complete {
            Theme::PROGRESS_COMPLETE
        } else if status == "PAUSED" {
            Theme::PROGRESS_PAUSED
        } else {
            Theme::PROGRESS_ACTIVE
        };

        Style::default().fg(color).bg(Theme::BACKGROUND)
    }

    /// Gauge style for progress bars
    pub fn gauge(progress: f64, status: &str) -> Style {
        let color = if progress >= 1.0 {
            Theme::PROGRESS_COMPLETE
        } else if status == "PAUSED" {
            Theme::PROGRESS_PAUSED
        } else {
            Theme::PROGRESS_ACTIVE
        };

        Style::default().fg(color).bg(Theme::BACKGROUND)
    }
}

/// Keyboard shortcut formatting
pub struct KeyStyle;

#[allow(dead_code)]
impl KeyStyle {
    /// Style for key labels in shortcuts
    pub fn key() -> Style {
        Style::default()
            .fg(Theme::SECONDARY)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for key descriptions
    pub fn description() -> Style {
        Style::default().fg(Theme::TEXT_MUTED)
    }
}
