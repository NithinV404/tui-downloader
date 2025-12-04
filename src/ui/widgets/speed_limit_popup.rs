//! Speed limit popup widget for setting download/upload speed limits

#![allow(dead_code)]

use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph},
    Frame,
};

/// Speed limit settings state
#[derive(Clone, Debug)]
pub struct SpeedLimitState {
    pub download_limit: u64,
    pub upload_limit: u64,
    pub editing_download: bool, // true = editing download, false = editing upload
    pub input_buffer: String,
}

impl Default for SpeedLimitState {
    fn default() -> Self {
        Self {
            download_limit: 0,
            upload_limit: 0,
            editing_download: true,
            input_buffer: String::new(),
        }
    }
}

impl SpeedLimitState {
    pub fn new(download_limit: u64, upload_limit: u64) -> Self {
        Self {
            download_limit,
            upload_limit,
            editing_download: true,
            input_buffer: String::new(),
        }
    }

    pub fn toggle_field(&mut self) {
        self.editing_download = !self.editing_download;
        self.input_buffer.clear();
    }

    pub fn get_current_limit(&self) -> u64 {
        if self.editing_download {
            self.download_limit
        } else {
            self.upload_limit
        }
    }

    pub fn set_current_limit(&mut self, limit: u64) {
        if self.editing_download {
            self.download_limit = limit;
        } else {
            self.upload_limit = limit;
        }
    }

    /// Increase limit by preset amount
    pub fn increase_limit(&mut self) {
        let current = self.get_current_limit();
        let new_limit = if current == 0 {
            1024 * 1024 // Start at 1 MB/s
        } else if current < 1024 * 1024 {
            // < 1 MB, increase by 100 KB
            current + 100 * 1024
        } else if current < 10 * 1024 * 1024 {
            // < 10 MB, increase by 1 MB
            current + 1024 * 1024
        } else {
            // >= 10 MB, increase by 5 MB
            current + 5 * 1024 * 1024
        };
        self.set_current_limit(new_limit);
    }

    /// Decrease limit by preset amount
    pub fn decrease_limit(&mut self) {
        let current = self.get_current_limit();
        let new_limit = if current <= 100 * 1024 {
            0 // Set to unlimited
        } else if current <= 1024 * 1024 {
            // <= 1 MB, decrease by 100 KB
            current.saturating_sub(100 * 1024)
        } else if current <= 10 * 1024 * 1024 {
            // <= 10 MB, decrease by 1 MB
            current.saturating_sub(1024 * 1024)
        } else {
            // > 10 MB, decrease by 5 MB
            current.saturating_sub(5 * 1024 * 1024)
        };
        self.set_current_limit(new_limit);
    }

    /// Parse and apply input buffer to current field
    pub fn apply_input(&mut self) -> bool {
        if let Some(limit) = parse_speed_limit(&self.input_buffer) {
            self.set_current_limit(limit);
            self.input_buffer.clear();
            true
        } else {
            false
        }
    }
}

/// Render the speed limit popup
pub fn render(f: &mut Frame, area: Rect, state: &SpeedLimitState) {
    // Calculate popup size (centered, 50% width, 40% height)
    let popup_area = centered_rect(50, 40, area);

    // Clear the area behind the popup
    f.render_widget(Clear, popup_area);

    // Create the popup block with rounded borders
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(" Speed Limits ")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Theme::WARNING));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Layout: Title, Download, Upload, Input, Footer
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Description
            Constraint::Length(3), // Download limit
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Upload limit
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Input field
            Constraint::Min(1),    // Footer
        ])
        .split(inner);

    // Render description
    let desc = Paragraph::new(vec![Line::from(vec![Span::styled(
        "Set bandwidth limits (0 = unlimited)",
        Style::default().fg(Theme::TEXT_MUTED),
    )])])
    .alignment(Alignment::Center);
    f.render_widget(desc, layout[0]);

    // Render download limit
    render_limit_field(
        f,
        layout[1],
        "Download",
        state.download_limit,
        state.editing_download,
        &state.input_buffer,
        true,
    );

    // Render upload limit
    render_limit_field(
        f,
        layout[3],
        "Upload",
        state.upload_limit,
        !state.editing_download,
        &state.input_buffer,
        false,
    );

    // Render input hint
    let input_hint = if !state.input_buffer.is_empty() {
        Line::from(vec![
            Span::styled("  Input: ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(
                state.input_buffer.clone(),
                Style::default()
                    .fg(Theme::HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " (e.g., 5m, 500k, 0)",
                Style::default().fg(Theme::TEXT_MUTED),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                "  Type a value (e.g., ",
                Style::default().fg(Theme::TEXT_MUTED),
            ),
            Span::styled(
                "5m",
                Style::default()
                    .fg(Theme::HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(", ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(
                "500k",
                Style::default()
                    .fg(Theme::HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(", ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(
                "0",
                Style::default()
                    .fg(Theme::HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" for unlimited)", Style::default().fg(Theme::TEXT_MUTED)),
        ])
    };
    f.render_widget(Paragraph::new(input_hint), layout[5]);

    // Render footer
    let footer = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "^/v",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" switch  ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            "</> ",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" adjust  ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" apply  ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" cancel", Style::default().fg(Theme::TEXT_MUTED)),
    ])])
    .alignment(Alignment::Center);
    f.render_widget(footer, layout[6]);
}

/// Render a speed limit field with gauge
fn render_limit_field(
    f: &mut Frame,
    area: Rect,
    label: &str,
    limit: u64,
    is_selected: bool,
    input_buffer: &str,
    is_download: bool,
) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12), // Label
            Constraint::Min(10),    // Gauge
            Constraint::Length(15), // Value
        ])
        .split(area);

    // Label with arrow indicator
    let label_style = if is_selected {
        Style::default()
            .fg(Theme::HIGHLIGHT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::TEXT_MUTED)
    };

    let arrow = if is_download { "v" } else { "^" };
    let indicator = if is_selected { ">> " } else { "   " };

    let label_line = Line::from(vec![
        Span::styled(indicator, label_style),
        Span::styled(
            arrow,
            if is_download {
                Style::default().fg(Theme::SUCCESS)
            } else {
                Style::default().fg(Theme::INFO)
            },
        ),
        Span::styled(format!(" {}", label), label_style),
    ]);
    f.render_widget(Paragraph::new(label_line), layout[0]);

    // Gauge showing relative limit (max 100 MB/s for visualization)
    let max_for_gauge = 100 * 1024 * 1024; // 100 MB/s
    let ratio = if limit == 0 {
        1.0 // Unlimited shows as full
    } else {
        (limit as f64 / max_for_gauge as f64).min(1.0)
    };

    let gauge_style = if is_selected {
        Style::default().fg(Theme::HIGHLIGHT).bg(Theme::BACKGROUND)
    } else if is_download {
        Style::default().fg(Theme::SUCCESS).bg(Theme::BACKGROUND)
    } else {
        Style::default().fg(Theme::INFO).bg(Theme::BACKGROUND)
    };

    let gauge = Gauge::default()
        .gauge_style(gauge_style)
        .ratio(ratio)
        .label("");

    f.render_widget(gauge, layout[1]);

    // Value display
    let value_text = if is_selected && !input_buffer.is_empty() {
        format!("{}_", input_buffer)
    } else {
        format_speed_limit(limit)
    };

    let value_style = if is_selected {
        Style::default()
            .fg(Theme::HIGHLIGHT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::CMD_COLOR)
    };

    let value_paragraph = Paragraph::new(Line::from(vec![Span::styled(value_text, value_style)]))
        .alignment(Alignment::Right);
    f.render_widget(value_paragraph, layout[2]);
}

/// Format speed limit for display
pub fn format_speed_limit(limit: u64) -> String {
    if limit == 0 {
        "Unlimited".to_string()
    } else {
        format_speed(limit)
    }
}

/// Format speed in human-readable format
fn format_speed(speed_bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if speed_bytes >= GB {
        format!("{:.1} GB/s", speed_bytes as f64 / GB as f64)
    } else if speed_bytes >= MB {
        format!("{:.1} MB/s", speed_bytes as f64 / MB as f64)
    } else if speed_bytes >= KB {
        format!("{:.0} KB/s", speed_bytes as f64 / KB as f64)
    } else {
        format!("{} B/s", speed_bytes)
    }
}

/// Parse speed limit from user input
pub fn parse_speed_limit(input: &str) -> Option<u64> {
    let input = input.trim().to_lowercase();

    if input.is_empty() || input == "0" || input == "unlimited" || input == "none" {
        return Some(0);
    }

    // Parse formats like "5m", "5mb", "5 mb/s", "5000k", etc.
    let mut num_str = String::new();
    let mut unit_str = String::new();
    let mut in_unit = false;

    for c in input.chars() {
        if c.is_ascii_digit() || c == '.' {
            if !in_unit {
                num_str.push(c);
            }
        } else if c.is_alphabetic() {
            in_unit = true;
            unit_str.push(c);
        }
    }

    let num: f64 = num_str.parse().ok()?;

    let multiplier: u64 = if unit_str.starts_with('g') {
        1024 * 1024 * 1024
    } else if unit_str.starts_with('m') {
        1024 * 1024
    } else if unit_str.starts_with('k') {
        1024
    } else if unit_str.is_empty() {
        // Assume MB/s if no unit
        1024 * 1024
    } else {
        1 // bytes
    };

    Some((num * multiplier as f64) as u64)
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_speed_limit() {
        assert_eq!(parse_speed_limit("5m"), Some(5 * 1024 * 1024));
        assert_eq!(parse_speed_limit("5mb"), Some(5 * 1024 * 1024));
        assert_eq!(parse_speed_limit("5 MB/s"), Some(5 * 1024 * 1024));
        assert_eq!(parse_speed_limit("500k"), Some(500 * 1024));
        assert_eq!(parse_speed_limit("500kb"), Some(500 * 1024));
        assert_eq!(parse_speed_limit("1g"), Some(1024 * 1024 * 1024));
        assert_eq!(parse_speed_limit("0"), Some(0));
        assert_eq!(parse_speed_limit("unlimited"), Some(0));
        assert_eq!(parse_speed_limit(""), Some(0));
        assert_eq!(parse_speed_limit("5"), Some(5 * 1024 * 1024)); // Assume MB
    }

    #[test]
    fn test_format_speed_limit() {
        assert_eq!(format_speed_limit(0), "Unlimited");
        assert_eq!(format_speed_limit(1024 * 1024), "1.0 MB/s");
        assert_eq!(format_speed_limit(500 * 1024), "500 KB/s");
        assert_eq!(format_speed_limit(1024 * 1024 * 1024), "1.0 GB/s");
    }

    #[test]
    fn test_speed_limit_state() {
        let mut state = SpeedLimitState::new(0, 0);
        assert!(state.editing_download);

        state.toggle_field();
        assert!(!state.editing_download);

        state.toggle_field();
        assert!(state.editing_download);
    }

    #[test]
    fn test_increase_decrease_limit() {
        let mut state = SpeedLimitState::new(0, 0);

        state.increase_limit();
        assert_eq!(state.download_limit, 1024 * 1024); // Should start at 1 MB

        state.increase_limit();
        assert_eq!(state.download_limit, 2 * 1024 * 1024); // Should be 2 MB

        state.decrease_limit();
        assert_eq!(state.download_limit, 1024 * 1024); // Back to 1 MB

        state.download_limit = 500 * 1024; // Set to 500 KB
        state.decrease_limit();
        assert_eq!(state.download_limit, 400 * 1024); // Should decrease by 100 KB
    }

    #[test]
    fn test_apply_input() {
        let mut state = SpeedLimitState::new(0, 0);
        state.input_buffer = "5m".to_string();

        assert!(state.apply_input());
        assert_eq!(state.download_limit, 5 * 1024 * 1024);
        assert!(state.input_buffer.is_empty());
    }

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(50, 50, area);

        assert!(centered.x >= 20 && centered.x <= 30);
        assert!(centered.y >= 20 && centered.y <= 30);
    }
}
