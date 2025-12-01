//! Details panel widget for displaying download information

use crate::models::Download;
use crate::ui::theme::Styles;
use crate::ui::utils::{download_type_name, format_size, format_speed};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Sparkline},
};

/// Render the complete details panel
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Area to render in
/// * `download` - Download to display details for
pub fn render(f: &mut Frame, area: Rect, download: &Download) {
    let has_error = download.status == "ERROR" || download.status.contains("error");

    // Create main container with rounded border for floating effect
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(if has_error {
            "━━ Download Details ⚠ ━━"
        } else {
            "━━ Download Details ━━"
        })
        .border_style(if has_error {
            Styles::error()
        } else {
            Styles::border()
        });

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Main vertical layout with spacing
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top padding
            Constraint::Length(7), // File info
            Constraint::Length(1), // Spacing
            Constraint::Length(1), // Progress label
            Constraint::Length(3), // Progress bar (with border)
            Constraint::Length(1), // Spacing
            Constraint::Length(5), // Download speed graph
            Constraint::Length(1), // Spacing
            Constraint::Length(5), // Upload speed graph
            Constraint::Length(1), // Spacing
            Constraint::Min(1),    // Additional info
            Constraint::Length(1), // Bottom padding
        ])
        .split(inner);

    render_info_section(f, layout[1], download, has_error);
    render_progress_label(f, layout[3], download);
    render_progress_bar(f, layout[4], download);
    render_download_speed_graph(f, layout[6], download);
    render_upload_speed_graph(f, layout[8], download);
    render_additional_info(f, layout[10], download);
}

/// Render download information section
fn render_info_section(f: &mut Frame, area: Rect, download: &Download, has_error: bool) {
    let mut info_lines = vec![
        Line::from(vec![
            Span::styled("  Status: ", Styles::text_muted()),
            Span::styled(&download.status, Styles::status(&download.status)),
        ]),
        Line::from(""),
    ];

    // Show error details if present
    if has_error {
        if let Some(error_msg) = &download.error_message {
            info_lines.push(Line::from(vec![
                Span::styled("  ⚠ Error: ", Styles::error()),
                Span::styled(error_msg, Styles::error()),
            ]));
        } else {
            info_lines.push(Line::from(vec![
                Span::styled("  ⚠ Error: ", Styles::error()),
                Span::styled(
                    "Download failed. Check connection or file.",
                    Styles::text_muted(),
                ),
            ]));
        }
        info_lines.push(Line::from(""));
    }

    info_lines.extend(vec![
        Line::from(vec![
            Span::styled("  File: ", Styles::text_muted()),
            Span::styled(&download.name, Styles::text()),
        ]),
        Line::from(vec![
            Span::styled("  Type: ", Styles::text_muted()),
            Span::styled(download_type_name(download), Styles::text()),
            Span::styled("  │  Size: ", Styles::text_muted()),
            Span::styled(
                format!(
                    "{} / {}",
                    format_size(download.completed_length),
                    if download.total_length > 0 {
                        format_size(download.total_length)
                    } else {
                        "Unknown".to_string()
                    }
                ),
                Styles::text(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ↓ ", Styles::success()),
            Span::styled(
                &download.speed,
                if has_error {
                    Styles::error()
                } else {
                    Styles::success()
                },
            ),
            Span::styled("  │  ↑ ", Styles::info()),
            Span::styled(&download.upload_speed, Styles::info()),
            Span::styled("  │  Conn: ", Styles::text_muted()),
            Span::styled(download.connections.to_string(), Styles::text_muted()),
        ]),
    ]);

    let paragraph = Paragraph::new(info_lines);
    f.render_widget(paragraph, area);
}

/// Render progress label
fn render_progress_label(f: &mut Frame, area: Rect, download: &Download) {
    let label = Line::from(vec![
        Span::styled("  Progress: ", Styles::text_muted()),
        Span::styled(
            format!("{:.1}%", download.progress * 100.0),
            Styles::status(&download.status),
        ),
    ]);

    let paragraph = Paragraph::new(label);
    f.render_widget(paragraph, area);
}

/// Render progress bar
fn render_progress_bar(f: &mut Frame, area: Rect, download: &Download) {
    let label = format!("{:.1}%", download.progress * 100.0);

    // Add padding to align with rest of content
    let padded_area = Rect {
        x: area.x + 2,
        y: area.y,
        width: area.width.saturating_sub(4),
        height: area.height,
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Styles::border()),
        )
        .gauge_style(Styles::gauge(download.progress, &download.status))
        .percent((download.progress * 100.0) as u16)
        .label(label);

    f.render_widget(gauge, padded_area);
}

/// Render download speed history graph
fn render_download_speed_graph(f: &mut Frame, area: Rect, download: &Download) {
    if !download.speed_history.is_empty() {
        let speed_data: Vec<u64> = download.speed_history.iter().cloned().collect();
        let max_speed = speed_data.iter().max().cloned().unwrap_or(1);
        let avg_speed = speed_data.iter().sum::<u64>() / speed_data.len() as u64;
        let max_speed_str = format_speed(max_speed);

        // Determine if speed is too low (< 100 KB/s average)
        let is_low_speed = avg_speed < 102400; // 100 KB in bytes
        let speed_style = if is_low_speed {
            Styles::warning()
        } else {
            Styles::success()
        };

        // Title line
        let title = Line::from(vec![
            Span::styled("  ↓ Download ", Styles::text_muted()),
            Span::styled("│ ", Styles::text_muted()),
            Span::styled("Peak: ", Styles::text_muted()),
            Span::styled(max_speed_str, speed_style),
        ]);

        let title_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        let separator_area = Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: 1,
        };
        let graph_area = Rect {
            x: area.x + 2,
            y: area.y + 2,
            width: area.width.saturating_sub(4),
            height: area.height.saturating_sub(2),
        };

        f.render_widget(Paragraph::new(title), title_area);
        f.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                "  ━".repeat((area.width as usize).saturating_sub(4) / 2),
                Styles::text_muted(),
            )])),
            separator_area,
        );

        let sparkline = Sparkline::default()
            .data(&speed_data)
            .style(speed_style)
            .max(max_speed);

        f.render_widget(sparkline, graph_area);
    } else {
        let no_data = Paragraph::new(vec![
            Line::from(vec![Span::styled("  ↓ Download", Styles::text_muted())]),
            Line::from(vec![Span::styled("  ━━━━━━━━━━━━━━", Styles::text_muted())]),
            Line::from(vec![Span::styled("  No data yet...", Styles::text_muted())]),
        ]);

        f.render_widget(no_data, area);
    }
}

/// Render upload speed history graph
fn render_upload_speed_graph(f: &mut Frame, area: Rect, download: &Download) {
    if !download.upload_speed_history.is_empty() {
        let speed_data: Vec<u64> = download.upload_speed_history.iter().cloned().collect();
        let max_speed = speed_data.iter().max().cloned().unwrap_or(1);
        let avg_speed = speed_data.iter().sum::<u64>() / speed_data.len() as u64;
        let max_speed_str = format_speed(max_speed);

        // Determine if speed is too low (< 50 KB/s average for upload)
        let is_low_speed = avg_speed < 51200; // 50 KB in bytes
        let speed_style = if is_low_speed {
            Styles::warning()
        } else {
            Styles::info()
        };

        // Title line
        let title = Line::from(vec![
            Span::styled("  ↑ Upload ", Styles::text_muted()),
            Span::styled("│ ", Styles::text_muted()),
            Span::styled("Peak: ", Styles::text_muted()),
            Span::styled(max_speed_str, speed_style),
        ]);

        let title_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        let separator_area = Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: 1,
        };
        let graph_area = Rect {
            x: area.x + 2,
            y: area.y + 2,
            width: area.width.saturating_sub(4),
            height: area.height.saturating_sub(2),
        };

        f.render_widget(Paragraph::new(title), title_area);
        f.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                "  ━".repeat((area.width as usize).saturating_sub(4) / 2),
                Styles::text_muted(),
            )])),
            separator_area,
        );

        let sparkline = Sparkline::default()
            .data(&speed_data)
            .style(speed_style)
            .max(max_speed);

        f.render_widget(sparkline, graph_area);
    } else {
        let no_data = Paragraph::new(vec![
            Line::from(vec![Span::styled("  ↑ Upload", Styles::text_muted())]),
            Line::from(vec![Span::styled("  ━━━━━━━━━━━━━━", Styles::text_muted())]),
            Line::from(vec![Span::styled("  No data yet...", Styles::text_muted())]),
        ]);

        f.render_widget(no_data, area);
    }
}

/// Render additional information
fn render_additional_info(f: &mut Frame, area: Rect, download: &Download) {
    let mut info_lines = vec![
        Line::from(vec![Span::styled(
            "  ━━━━━━━━━━━━━━━━━━━━",
            Styles::text_muted(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  GID: ", Styles::text_muted()),
            Span::styled(
                download.gid.as_ref().map(|s| s.as_str()).unwrap_or("N/A"),
                Styles::text(),
            ),
        ]),
    ];

    if let Some(path) = &download.file_path {
        info_lines.push(Line::from(""));
        info_lines.push(Line::from(vec![
            Span::styled("  Path: ", Styles::text_muted()),
            Span::styled(path.as_str(), Styles::text()),
        ]));
    } else {
        // Show default download directory
        let download_dir = dirs::download_dir()
            .or_else(|| dirs::home_dir().map(|p| p.join("Downloads")))
            .unwrap_or_else(|| std::path::PathBuf::from("./Downloads"));
        info_lines.push(Line::from(""));
        info_lines.push(Line::from(vec![
            Span::styled("  Path: ", Styles::text_muted()),
            Span::styled(download_dir.display().to_string(), Styles::text()),
        ]));
    }

    if let Some(url) = &download.url {
        info_lines.push(Line::from(""));
        info_lines.push(Line::from(vec![Span::styled(
            "  URL: ",
            Styles::text_muted(),
        )]));
        info_lines.push(Line::from(vec![
            Span::styled("  ", Styles::text()),
            Span::styled(url.as_str(), Styles::text()),
        ]));
    }

    use crate::models::DownloadType;
    if download.download_type == DownloadType::Torrent {
        info_lines.push(Line::from(""));
        info_lines.push(Line::from(vec![
            Span::styled("  ℹ ", Styles::info()),
            Span::styled(
                "BitTorrent: DHT, Peer Exchange, Local Discovery enabled",
                Styles::text_muted(),
            ),
        ]));
    }

    // Add helpful tip
    info_lines.push(Line::from(""));
    info_lines.push(Line::from(vec![
        Span::styled("  💡 ", Styles::warning()),
        Span::styled(
            "[Space] pause/resume  │  [D] delete  │  [X] purge completed",
            Styles::text_muted(),
        ),
    ]));

    let paragraph = Paragraph::new(info_lines);
    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DownloadType;

    fn create_test_download() -> Download {
        Download {
            gid: Some("test123".to_string()),
            name: "test_file.zip".to_string(),
            url: Some("https://example.com/file.zip".to_string()),
            progress: 0.5,
            speed: "5.2 MB/s".to_string(),
            status: "ACTIVE".to_string(),
            total_length: 1024 * 1024 * 100,
            completed_length: 1024 * 1024 * 50,
            download_type: DownloadType::Http,
            speed_history: vec![1024, 2048, 4096, 8192],
            upload_speed: "50 KB/s".to_string(),
            upload_speed_history: vec![512, 1024, 2048, 1024],
            connections: 4,
            file_path: Some("/downloads/test_file.zip".to_string()),
            error_message: None,
        }
    }

    #[test]
    fn test_download_creation() {
        let download = create_test_download();
        assert_eq!(download.name, "test_file.zip");
        assert_eq!(download.progress, 0.5);
        assert_eq!(download.connections, 4);
    }
}
