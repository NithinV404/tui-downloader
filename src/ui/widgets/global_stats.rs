//! Global statistics bar widget showing aggregate download information

#![allow(dead_code)]

use crate::ui::theme::Styles;
use crate::ui::utils::GlobalStats;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render the global statistics bar (minimal, no heavy borders)
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Area to render in
/// * `stats` - Global statistics to display
/// * `download_limit` - Current download speed limit (0 = unlimited)
/// * `upload_limit` - Current upload speed limit (0 = unlimited)
pub fn render(
    f: &mut Frame,
    area: Rect,
    stats: &GlobalStats,
    download_limit: u64,
    upload_limit: u64,
) {
    // Single line of stats with subtle separator below
    let stats_line = build_stats_line(stats, download_limit, upload_limit);
    let paragraph = Paragraph::new(stats_line).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

/// Render a compact version of the stats bar (single line, no border)
pub fn render_compact(
    f: &mut Frame,
    area: Rect,
    stats: &GlobalStats,
    download_limit: u64,
    upload_limit: u64,
) {
    let stats_line = build_stats_line(stats, download_limit, upload_limit);
    let paragraph = Paragraph::new(stats_line).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

/// Render an expanded version with more details
pub fn render_expanded(
    f: &mut Frame,
    area: Rect,
    stats: &GlobalStats,
    download_limit: u64,
    upload_limit: u64,
) {
    // Layout for two rows
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    // First row: Speed info
    let speed_line = build_speed_line(stats, download_limit, upload_limit);
    f.render_widget(
        Paragraph::new(speed_line).alignment(Alignment::Center),
        layout[0],
    );

    // Second row: Count info
    let count_line = build_count_line(stats);
    f.render_widget(
        Paragraph::new(count_line).alignment(Alignment::Center),
        layout[1],
    );
}

/// Build the main stats line
fn build_stats_line(stats: &GlobalStats, download_limit: u64, upload_limit: u64) -> Line<'static> {
    let mut spans = vec![];

    // Download speed with icon
    spans.push(Span::styled("D: ", Styles::success()));
    spans.push(Span::styled(
        format_speed(stats.total_download_speed),
        Styles::success(),
    ));

    // Download limit indicator
    if download_limit > 0 {
        spans.push(Span::styled(
            format!(" [{}]", format_speed_short(download_limit)),
            Styles::text_muted(),
        ));
    }

    spans.push(Span::styled("   ", Styles::text_muted()));

    // Upload speed with icon
    spans.push(Span::styled("U: ", Styles::info()));
    spans.push(Span::styled(
        format_speed(stats.total_upload_speed),
        Styles::info(),
    ));

    // Upload limit indicator
    if upload_limit > 0 {
        spans.push(Span::styled(
            format!(" [{}]", format_speed_short(upload_limit)),
            Styles::text_muted(),
        ));
    }

    spans.push(Span::styled("   ·   ", Styles::text_muted()));

    // Active count with subtle styling
    spans.push(Span::styled(
        format!("{}", stats.active_count),
        if stats.active_count > 0 {
            Styles::success()
        } else {
            Styles::text_muted()
        },
    ));
    spans.push(Span::styled(" active", Styles::text_muted()));

    spans.push(Span::styled("   ", Styles::text_muted()));

    // Waiting count
    spans.push(Span::styled(
        format!("{}", stats.waiting_count),
        if stats.waiting_count > 0 {
            Styles::warning()
        } else {
            Styles::text_muted()
        },
    ));
    spans.push(Span::styled(" queued", Styles::text_muted()));

    spans.push(Span::styled("   ", Styles::text_muted()));

    // Completed count
    spans.push(Span::styled(
        format!("{}", stats.completed_count),
        Styles::text_muted(),
    ));
    spans.push(Span::styled(" done", Styles::text_muted()));

    // Error count (only show if > 0)
    if stats.error_count > 0 {
        spans.push(Span::styled("   ", Styles::text_muted()));
        spans.push(Span::styled(
            format!("{}", stats.error_count),
            Styles::error(),
        ));
        spans.push(Span::styled(" errors", Styles::error()));
    }

    Line::from(spans)
}

/// Build the speed-focused line for expanded view
fn build_speed_line(stats: &GlobalStats, download_limit: u64, upload_limit: u64) -> Line<'static> {
    let mut spans = vec![];

    // Download speed with limit
    spans.push(Span::styled("D: ", Styles::success()));
    spans.push(Span::styled(
        format_speed(stats.total_download_speed),
        Styles::success(),
    ));

    if download_limit > 0 {
        spans.push(Span::styled(" / ", Styles::text_muted()));
        spans.push(Span::styled(
            format_speed(download_limit),
            Styles::text_muted(),
        ));
    }

    spans.push(Span::styled("      ", Styles::text_muted()));

    // Upload speed with limit
    spans.push(Span::styled("U: ", Styles::info()));
    spans.push(Span::styled(
        format_speed(stats.total_upload_speed),
        Styles::info(),
    ));

    if upload_limit > 0 {
        spans.push(Span::styled(" / ", Styles::text_muted()));
        spans.push(Span::styled(
            format_speed(upload_limit),
            Styles::text_muted(),
        ));
    }

    // Total progress
    if stats.total_size > 0 {
        let progress = (stats.total_downloaded as f64 / stats.total_size as f64) * 100.0;
        spans.push(Span::styled("      ", Styles::text_muted()));
        spans.push(Span::styled(
            format!("{:.1}%", progress),
            if progress >= 100.0 {
                Styles::success()
            } else {
                Styles::highlight()
            },
        ));
        spans.push(Span::styled(" total", Styles::text_muted()));
    }

    Line::from(spans)
}

/// Build the count-focused line for expanded view
fn build_count_line(stats: &GlobalStats) -> Line<'static> {
    let mut spans = vec![];

    // Active
    spans.push(Span::styled("● ", Styles::success()));
    spans.push(Span::styled(
        stats.active_count.to_string(),
        Styles::success(),
    ));
    spans.push(Span::styled(" active", Styles::text_muted()));

    spans.push(Span::styled("    ", Styles::text_muted()));

    // Waiting/Queued
    spans.push(Span::styled("○ ", Styles::warning()));
    spans.push(Span::styled(
        stats.waiting_count.to_string(),
        Styles::warning(),
    ));
    spans.push(Span::styled(" queued", Styles::text_muted()));

    spans.push(Span::styled("    ", Styles::text_muted()));

    // Completed
    spans.push(Span::styled("* ", Styles::info()));
    spans.push(Span::styled(
        stats.completed_count.to_string(),
        Styles::info(),
    ));
    spans.push(Span::styled(" done", Styles::text_muted()));

    // Errors (only if present)
    if stats.error_count > 0 {
        spans.push(Span::styled("    ", Styles::text_muted()));
        spans.push(Span::styled("x ", Styles::error()));
        spans.push(Span::styled(stats.error_count.to_string(), Styles::error()));
        spans.push(Span::styled(" errors", Styles::error()));
    }

    // Total downloaded
    if stats.total_downloaded > 0 {
        spans.push(Span::styled("    ·    ", Styles::text_muted()));
        spans.push(Span::styled(
            format_size(stats.total_downloaded),
            Styles::text(),
        ));

        if stats.total_size > 0 && stats.total_size > stats.total_downloaded {
            spans.push(Span::styled(" / ", Styles::text_muted()));
            spans.push(Span::styled(
                format_size(stats.total_size),
                Styles::text_muted(),
            ));
        }
    }

    Line::from(spans)
}

/// Format speed in human-readable format
fn format_speed(speed_bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if speed_bytes >= GB {
        format!("{:.2} GB/s", speed_bytes as f64 / GB as f64)
    } else if speed_bytes >= MB {
        format!("{:.2} MB/s", speed_bytes as f64 / MB as f64)
    } else if speed_bytes >= KB {
        format!("{:.1} KB/s", speed_bytes as f64 / KB as f64)
    } else {
        format!("{} B/s", speed_bytes)
    }
}

/// Format speed in short format (for limit display)
fn format_speed_short(speed_bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if speed_bytes >= GB {
        format!("{:.0}G", speed_bytes as f64 / GB as f64)
    } else if speed_bytes >= MB {
        format!("{:.0}M", speed_bytes as f64 / MB as f64)
    } else if speed_bytes >= KB {
        format!("{:.0}K", speed_bytes as f64 / KB as f64)
    } else {
        format!("{}B", speed_bytes)
    }
}

/// Format file size in human-readable format
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(0), "0 B/s");
        assert_eq!(format_speed(512), "512 B/s");
        assert_eq!(format_speed(1024), "1.0 KB/s");
        assert_eq!(format_speed(1536), "1.5 KB/s");
        assert_eq!(format_speed(1048576), "1.00 MB/s");
        assert_eq!(format_speed(1073741824), "1.00 GB/s");
    }

    #[test]
    fn test_format_speed_short() {
        assert_eq!(format_speed_short(0), "0B");
        assert_eq!(format_speed_short(1024), "1K");
        assert_eq!(format_speed_short(1048576), "1M");
        assert_eq!(format_speed_short(1073741824), "1G");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
        assert_eq!(format_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_build_stats_line() {
        let stats = GlobalStats {
            total_download_speed: 1048576,
            total_upload_speed: 524288,
            active_count: 2,
            waiting_count: 5,
            completed_count: 10,
            error_count: 0,
            total_downloaded: 0,
            total_size: 0,
        };

        let line = build_stats_line(&stats, 0, 0);
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_build_stats_line_with_limits() {
        let stats = GlobalStats::default();
        let line = build_stats_line(&stats, 5 * 1024 * 1024, 1024 * 1024);

        // Should contain limit indicators
        let text: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(text.contains("[5M]"));
        assert!(text.contains("[1M]"));
    }

    #[test]
    fn test_build_stats_line_with_errors() {
        let stats = GlobalStats {
            total_download_speed: 0,
            total_upload_speed: 0,
            active_count: 0,
            waiting_count: 0,
            completed_count: 5,
            error_count: 2,
            total_downloaded: 0,
            total_size: 0,
        };

        let line = build_stats_line(&stats, 0, 0);
        let text: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(text.contains("errors"));
        assert!(text.contains("2"));
    }

    #[test]
    fn test_global_stats_default() {
        let stats = GlobalStats::default();
        assert_eq!(stats.active_count, 0);
        assert_eq!(stats.waiting_count, 0);
        assert_eq!(stats.completed_count, 0);
        assert_eq!(stats.error_count, 0);
        assert_eq!(stats.total_download_speed, 0);
        assert_eq!(stats.total_upload_speed, 0);
    }
}
