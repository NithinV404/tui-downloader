//! Utility functions for UI operations

use crate::models::Download;

/// Filter downloads based on tab index
///
/// # Arguments
/// * `downloads` - Slice of downloads to filter
/// * `tab` - Tab index (0 = Active, 1 = Queue, 2 = Completed)
///
/// # Returns
/// Vector of references to downloads matching the tab criteria
pub fn filter_by_tab(downloads: &[Download], tab: usize) -> Vec<&Download> {
    downloads
        .iter()
        .filter(|d| match tab {
            0 => is_active(d),
            1 => is_queued(d),
            2 => is_completed(d),
            _ => false,
        })
        .collect()
}

/// Check if a download is active
pub fn is_active(download: &Download) -> bool {
    download.status == "ACTIVE"
        || (download.progress > 0.0 && download.progress < 1.0 && download.status != "WAITING")
}

/// Check if a download is queued
pub fn is_queued(download: &Download) -> bool {
    download.status == "WAITING" || download.status == "PAUSED" || download.progress == 0.0
}

/// Check if a download is completed
pub fn is_completed(download: &Download) -> bool {
    download.progress >= 1.0 || download.status == "COMPLETE"
}

/// Count downloads by tab
pub fn count_by_tab(downloads: &[Download], tab: usize) -> usize {
    downloads
        .iter()
        .filter(|d| match tab {
            0 => is_active(d),
            1 => is_queued(d),
            2 => is_completed(d),
            _ => false,
        })
        .count()
}

/// Format file size in human-readable format
pub fn format_size(bytes: u64) -> String {
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
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format speed in human-readable format
pub fn format_speed(speed_bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if speed_bytes >= GB {
        format!("{:.2} GB/s", speed_bytes as f64 / GB as f64)
    } else if speed_bytes >= MB {
        format!("{:.2} MB/s", speed_bytes as f64 / MB as f64)
    } else if speed_bytes >= KB {
        format!("{:.2} KB/s", speed_bytes as f64 / KB as f64)
    } else {
        format!("{} B/s", speed_bytes)
    }
}

/// Truncate text with ellipsis if too long
pub fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() > max_len {
        format!("{}...", &text[0..max_len.saturating_sub(3)])
    } else {
        text.to_string()
    }
}

/// Get download type display name
pub fn download_type_name(download: &Download) -> &'static str {
    use crate::models::DownloadType;

    match download.download_type {
        DownloadType::Http => "HTTP/HTTPS",
        DownloadType::Torrent => "BitTorrent",
        DownloadType::Metalink => "Metalink",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Download, DownloadType};

    fn create_test_download(status: &str, progress: f64) -> Download {
        Download {
            gid: Some("test".to_string()),
            name: "test.txt".to_string(),
            url: None,
            progress: progress,
            speed: "0 B/s".to_string(),
            status: status.to_string(),
            total_length: 0,
            completed_length: 0,
            download_type: DownloadType::Http,
            speed_history: vec![],
            connections: 0,
            file_path: None,
            error_message: None,
        }
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
        assert_eq!(format_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(0), "0 B/s");
        assert_eq!(format_speed(1024), "1.00 KB/s");
        assert_eq!(format_speed(1048576), "1.00 MB/s");
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("short", 10), "short");
        assert_eq!(truncate_text("this is a very long text", 10), "this is...");
    }

    #[test]
    fn test_is_active() {
        let download = create_test_download("ACTIVE", 0.5);
        assert!(is_active(&download));

        let download = create_test_download("WAITING", 0.0);
        assert!(!is_active(&download));
    }

    #[test]
    fn test_is_queued() {
        let download = create_test_download("WAITING", 0.0);
        assert!(is_queued(&download));

        let download = create_test_download("PAUSED", 0.5);
        assert!(is_queued(&download));
    }

    #[test]
    fn test_is_completed() {
        let download = create_test_download("COMPLETE", 1.0);
        assert!(is_completed(&download));

        let download = create_test_download("ACTIVE", 0.5);
        assert!(!is_completed(&download));
    }
}
