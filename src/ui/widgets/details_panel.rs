//! Details panel widget for displaying download information

use crate::models::{Download, DownloadType};
use crate::ui::theme::{Styles, Theme};
use crate::ui::utils::{download_type_name, format_download_eta, format_size};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame,
};

/// Render the complete details panel
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Area to render in
/// * `download` - Download to display details for
pub fn render(f: &mut Frame, area: Rect, download: &Download) {
    let has_error = download.status == "ERROR" || download.status.to_lowercase().contains("error");

    // Create main container with rounded border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(if has_error {
            " Details [!] "
        } else {
            " Details "
        })
        .border_style(if has_error {
            Style::default().fg(Theme::ERROR)
        } else {
            Style::default().fg(Theme::BORDER)
        });

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Check for empty/placeholder state
    if download.gid.is_none() || download.name == "No downloads" {
        render_empty_state(f, inner);
        return;
    }

    let is_torrent = download.download_type == DownloadType::Torrent;

    // Check if we have piece data to show
    let has_pieces = download.num_pieces > 0 && download.bitfield.is_some();

    // Simplified layout - info section now includes seeds/peers for torrents
    let info_height = if is_torrent { 6 } else { 5 };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if has_pieces {
            vec![
                Constraint::Length(info_height), // File info section (includes seeds/peers for torrents)
                Constraint::Length(3),           // Progress box
                Constraint::Length(4),           // Pieces visualization box
                Constraint::Length(5),           // Download speed box
                Constraint::Length(5),           // Upload speed box
                Constraint::Min(1),              // Additional info
            ]
        } else {
            vec![
                Constraint::Length(info_height), // File info section
                Constraint::Length(3),           // Progress box
                Constraint::Length(5),           // Download speed box
                Constraint::Length(5),           // Upload speed box
                Constraint::Min(1),              // Additional info
            ]
        })
        .split(inner);

    render_info_section(f, layout[0], download, has_error, is_torrent);
    render_progress_box(f, layout[1], download, has_error);

    if has_pieces {
        render_pieces_box(f, layout[2], download);
        render_download_speed_box(f, layout[3], download);
        render_upload_speed_box(f, layout[4], download);
        render_additional_info(f, layout[5], download);
    } else {
        render_download_speed_box(f, layout[2], download);
        render_upload_speed_box(f, layout[3], download);
        render_additional_info(f, layout[4], download);
    }
}

/// Render empty state when no download is selected
fn render_empty_state(f: &mut Frame, area: Rect) {
    let message = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "No download selected",
            Styles::text_muted(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Styles::text_muted()),
            Span::styled("i", Styles::highlight()),
            Span::styled(" to add a download", Styles::text_muted()),
        ]),
    ]);

    f.render_widget(message, area);
}

/// Render download information section (compact, includes seeds/peers inline for torrents)
fn render_info_section(
    f: &mut Frame,
    area: Rect,
    download: &Download,
    has_error: bool,
    is_torrent: bool,
) {
    let mut info_lines = vec![];

    // Status line with icon (no emoji, simple characters)
    let status_icon = match download.status.as_str() {
        "ACTIVE" => ">",
        "PAUSED" => "||",
        "WAITING" => "o",
        "COMPLETE" => "*",
        "ERROR" => "x",
        _ => "-",
    };

    info_lines.push(Line::from(vec![
        Span::styled(
            format!(" {} ", status_icon),
            Styles::status(&download.status),
        ),
        Span::styled(&download.status, Styles::status(&download.status)),
        Span::styled("  ETA: ", Styles::text_muted()),
        Span::styled(
            format_download_eta(download),
            if download.progress >= 1.0 {
                Styles::success()
            } else {
                Styles::text_muted()
            },
        ),
    ]));

    // Error message if present
    if has_error {
        let error_msg = download
            .error_message
            .as_deref()
            .unwrap_or("Download failed");
        info_lines.push(Line::from(vec![
            Span::styled(" ", Styles::text_muted()),
            Span::styled(error_msg, Styles::error()),
        ]));
    }

    // File name (truncated if needed)
    let max_name_len = area.width.saturating_sub(4) as usize;
    let display_name = if download.name.len() > max_name_len {
        format!("{}...", &download.name[..max_name_len.saturating_sub(3)])
    } else {
        download.name.clone()
    };

    info_lines.push(Line::from(vec![
        Span::styled(" ", Styles::text_muted()),
        Span::styled(display_name, Styles::text()),
    ]));

    // Type and size on same line
    info_lines.push(Line::from(vec![
        Span::styled(" ", Styles::text_muted()),
        Span::styled(download_type_name(download), Styles::text_muted()),
        Span::styled(" | ", Styles::text_muted()),
        Span::styled(
            format!(
                "{} / {}",
                format_size(download.completed_length),
                if download.total_length > 0 {
                    format_size(download.total_length)
                } else {
                    "?".to_string()
                }
            ),
            Styles::text(),
        ),
    ]));

    // Seeds/Peers inline for torrents (dots indicator with numbers)
    if is_torrent {
        let seeds_dots = build_indicator_dots(download.seeds, 10);
        let peers_dots = build_indicator_dots(download.peers, 20);

        let seeds_style = if download.seeds > 0 {
            Styles::success()
        } else {
            Styles::warning()
        };

        let peers_style = if download.peers > 0 {
            Styles::info()
        } else {
            Styles::text_muted()
        };

        info_lines.push(Line::from(vec![
            Span::styled(" Seeds: ", Styles::text_muted()),
            Span::styled(seeds_dots, seeds_style),
            Span::styled(format!(" {}", download.seeds), seeds_style),
            Span::styled("  Peers: ", Styles::text_muted()),
            Span::styled(peers_dots, peers_style),
            Span::styled(format!(" {}", download.peers), peers_style),
        ]));
    }

    let paragraph = Paragraph::new(info_lines);
    f.render_widget(paragraph, area);
}

/// Build indicator dots based on count and max threshold
fn build_indicator_dots(count: u32, max_for_full: u32) -> String {
    let filled = if max_for_full == 0 {
        0
    } else {
        ((count as f64 / max_for_full as f64) * 5.0).ceil() as usize
    };
    let filled = filled.min(5);
    let empty = 5 - filled;

    let mut dots = String::new();
    for _ in 0..filled {
        dots.push('●');
    }
    for _ in 0..empty {
        dots.push('○');
    }
    dots
}

/// Render progress bar in a box
fn render_progress_box(f: &mut Frame, area: Rect, download: &Download, has_error: bool) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(" Progress ")
        .border_style(Style::default().fg(Theme::BORDER));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if has_error {
        let error_line = Line::from(vec![Span::styled("Error", Styles::error())]);
        f.render_widget(Paragraph::new(error_line), inner);
        return;
    }

    let label = format!("{:.1}%", download.progress * 100.0);

    let gauge_style = if download.progress >= 1.0 {
        Style::default().fg(Theme::STATUS_COMPLETE)
    } else if download.status == "PAUSED" {
        Style::default().fg(Theme::STATUS_PAUSED)
    } else {
        Style::default().fg(Theme::SUCCESS)
    };

    let gauge = Gauge::default()
        .ratio(download.progress)
        .label(label)
        .gauge_style(gauge_style);

    f.render_widget(gauge, inner);
}

/// Render download speed box with sparkline
fn render_download_speed_box(f: &mut Frame, area: Rect, download: &Download) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(format!(" Down: {} ", download.speed))
        .border_style(Style::default().fg(Theme::SUCCESS));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if download.speed_history.is_empty() {
        let no_data = Line::from(vec![Span::styled("No data yet", Styles::text_muted())]);
        f.render_widget(Paragraph::new(no_data), inner);
        return;
    }

    // Normalize data for sparkline
    let max_speed = download.speed_history.iter().max().copied().unwrap_or(1);
    let data: Vec<u64> = download
        .speed_history
        .iter()
        .map(|&s| {
            if max_speed > 0 {
                (s as f64 / max_speed as f64 * 64.0) as u64
            } else {
                0
            }
        })
        .collect();

    let sparkline = Sparkline::default()
        .data(&data)
        .style(Style::default().fg(Theme::SUCCESS));
    f.render_widget(sparkline, inner);
}

/// Render upload speed box with sparkline
fn render_upload_speed_box(f: &mut Frame, area: Rect, download: &Download) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(format!(" Up: {} ", download.upload_speed))
        .border_style(Style::default().fg(Theme::INFO));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if download.upload_speed_history.is_empty() {
        let no_data = Line::from(vec![Span::styled("No data yet", Styles::text_muted())]);
        f.render_widget(Paragraph::new(no_data), inner);
        return;
    }

    // Normalize data for sparkline
    let max_speed = download
        .upload_speed_history
        .iter()
        .max()
        .copied()
        .unwrap_or(1);
    let data: Vec<u64> = download
        .upload_speed_history
        .iter()
        .map(|&s| {
            if max_speed > 0 {
                (s as f64 / max_speed as f64 * 64.0) as u64
            } else {
                0
            }
        })
        .collect();

    let sparkline = Sparkline::default()
        .data(&data)
        .style(Style::default().fg(Theme::INFO));
    f.render_widget(sparkline, inner);
}

/// Render pieces/chunks visualization box
fn render_pieces_box(f: &mut Frame, area: Rect, download: &Download) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(format!(
            " Pieces [{}/{}] ",
            count_completed_pieces(download),
            download.num_pieces
        ))
        .border_style(Style::default().fg(Theme::BORDER));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if let Some(ref bitfield) = download.bitfield {
        let pieces_line =
            render_bitfield_visualization(bitfield, download.num_pieces, inner.width as usize);
        f.render_widget(Paragraph::new(pieces_line), inner);
    } else {
        let no_data = Line::from(vec![Span::styled("No piece data", Styles::text_muted())]);
        f.render_widget(Paragraph::new(no_data), inner);
    }
}

/// Count completed pieces from bitfield
fn count_completed_pieces(download: &Download) -> u32 {
    if let Some(ref bitfield) = download.bitfield {
        let mut count = 0u32;
        for c in bitfield.chars() {
            if let Some(val) = c.to_digit(16) {
                count += val.count_ones();
            }
        }
        count
    } else {
        0
    }
}

/// Render bitfield as a visual grid of blocks
fn render_bitfield_visualization(bitfield: &str, num_pieces: u32, width: usize) -> Line<'static> {
    if num_pieces == 0 || width == 0 {
        return Line::from(vec![Span::styled("No pieces", Styles::text_muted())]);
    }

    // Convert hex bitfield to a vector of booleans (true = have piece)
    let mut pieces: Vec<bool> = Vec::with_capacity(num_pieces as usize);
    for c in bitfield.chars() {
        if let Some(val) = c.to_digit(16) {
            // Each hex char represents 4 bits (pieces)
            for i in (0..4).rev() {
                if pieces.len() < num_pieces as usize {
                    pieces.push((val >> i) & 1 == 1);
                }
            }
        }
    }

    // Pad to num_pieces if needed
    while pieces.len() < num_pieces as usize {
        pieces.push(false);
    }

    // Calculate how many pieces each display block represents
    let display_width = width.saturating_sub(2).max(1); // Leave some margin
    let pieces_per_block = ((num_pieces as usize) + display_width - 1) / display_width;

    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::styled(" ", Styles::text_muted()));

    for block_idx in 0..display_width {
        let start = block_idx * pieces_per_block;
        let end = ((block_idx + 1) * pieces_per_block).min(pieces.len());

        if start >= pieces.len() {
            break;
        }

        // Calculate how many pieces in this block are complete
        let block_pieces = &pieces[start..end];
        let completed = block_pieces.iter().filter(|&&p| p).count();
        let total = block_pieces.len();

        // Choose character based on completion ratio
        let (ch, style) = if total == 0 {
            ('-', Style::default().fg(Theme::TEXT_MUTED))
        } else {
            let ratio = completed as f64 / total as f64;
            if ratio >= 1.0 {
                ('#', Style::default().fg(Theme::SUCCESS)) // Fully complete
            } else if ratio >= 0.75 {
                ('=', Style::default().fg(Theme::SUCCESS)) // Mostly complete
            } else if ratio >= 0.5 {
                ('+', Style::default().fg(Theme::WARNING)) // Half complete
            } else if ratio > 0.0 {
                ('.', Style::default().fg(Theme::WARNING)) // Partially complete
            } else {
                ('-', Style::default().fg(Theme::TEXT_MUTED)) // Empty
            }
        };

        spans.push(Span::styled(ch.to_string(), style));
    }

    Line::from(spans)
}

/// Render additional info section
fn render_additional_info(f: &mut Frame, area: Rect, download: &Download) {
    let mut info_lines = vec![];

    // Connections
    if download.connections > 0 {
        info_lines.push(Line::from(vec![
            Span::styled(" Connections: ", Styles::text_muted()),
            Span::styled(download.connections.to_string(), Styles::text()),
        ]));
    }

    // File path if available
    if let Some(path) = &download.file_path {
        let max_path_len = area.width.saturating_sub(10) as usize;
        let display_path = if path.len() > max_path_len {
            format!("...{}", &path[path.len().saturating_sub(max_path_len)..])
        } else {
            path.clone()
        };

        info_lines.push(Line::from(vec![
            Span::styled(" Path: ", Styles::text_muted()),
            Span::styled(display_path, Styles::text_muted()),
        ]));
    }

    // URL (truncated) if available
    if let Some(url) = &download.url {
        let max_url_len = area.width.saturating_sub(10) as usize;
        let display_url = if url.len() > max_url_len {
            format!("{}...", &url[..max_url_len.saturating_sub(3)])
        } else {
            url.clone()
        };

        info_lines.push(Line::from(vec![
            Span::styled(" URL: ", Styles::text_muted()),
            Span::styled(display_url, Styles::text_muted()),
        ]));
    }

    // Torrent-specific tips
    if download.download_type == DownloadType::Torrent && download.seeds == 0 {
        info_lines.push(Line::from(""));
        info_lines.push(Line::from(vec![Span::styled(
            " ! No seeds available",
            Styles::warning(),
        )]));
    }

    let paragraph = Paragraph::new(info_lines);
    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DownloadType;

    fn create_test_download(name: &str, status: &str, progress: f64) -> Download {
        Download {
            gid: Some("test123".to_string()),
            name: name.to_string(),
            url: Some("https://example.com/file.zip".to_string()),
            progress,
            speed: "1.5 MB/s".to_string(),
            status: status.to_string(),
            total_length: 1024 * 1024 * 100,
            completed_length: (1024 * 1024 * 100) as u64 * progress as u64 / 100,
            download_type: DownloadType::Http,
            speed_history: vec![1024 * 1024],
            upload_speed: "0 B/s".to_string(),
            upload_speed_history: vec![0],
            connections: 4,
            file_path: Some("/downloads/file.zip".to_string()),
            error_message: None,
            added_at: std::time::Instant::now(),
            seeds: 0,
            peers: 0,
            bitfield: None,
            num_pieces: 0,
        }
    }

    fn create_torrent_download() -> Download {
        let mut dl = create_test_download("ubuntu.torrent", "ACTIVE", 50.0);
        dl.download_type = DownloadType::Torrent;
        dl.seeds = 15;
        dl.peers = 42;
        dl
    }

    #[test]
    fn test_download_creation() {
        let download = create_test_download("test.zip", "ACTIVE", 0.5);
        assert_eq!(download.name, "test.zip");
        assert_eq!(download.status, "ACTIVE");
    }

    #[test]
    fn test_torrent_download() {
        let download = create_torrent_download();
        assert_eq!(download.download_type, DownloadType::Torrent);
        assert_eq!(download.seeds, 15);
        assert_eq!(download.peers, 42);
    }

    #[test]
    fn test_error_download() {
        let mut download = create_test_download("test.zip", "ERROR", 0.0);
        download.error_message = Some("Connection refused".to_string());
        assert_eq!(download.status, "ERROR");
        assert!(download.error_message.is_some());
    }

    #[test]
    fn test_completed_download() {
        let download = create_test_download("test.zip", "COMPLETE", 1.0);
        assert_eq!(download.status, "COMPLETE");
        assert_eq!(download.progress, 1.0);
    }

    #[test]
    fn test_paused_download() {
        let download = create_test_download("test.zip", "PAUSED", 0.5);
        assert_eq!(download.status, "PAUSED");
    }

    #[test]
    fn test_build_indicator_dots() {
        assert_eq!(build_indicator_dots(0, 10), "○○○○○");
        assert_eq!(build_indicator_dots(5, 10), "●●●○○");
        assert_eq!(build_indicator_dots(10, 10), "●●●●●");
        assert_eq!(build_indicator_dots(20, 10), "●●●●●");
    }

    #[test]
    fn test_count_completed_pieces() {
        let mut download = create_test_download("test.zip", "ACTIVE", 0.5);
        download.bitfield = Some("ff".to_string()); // 8 bits set
        download.num_pieces = 8;
        assert_eq!(count_completed_pieces(&download), 8);

        download.bitfield = Some("f0".to_string()); // 4 bits set
        assert_eq!(count_completed_pieces(&download), 4);

        download.bitfield = None;
        assert_eq!(count_completed_pieces(&download), 0);
    }
}
