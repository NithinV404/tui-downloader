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

/// Filter downloads by search query (case-insensitive name matching)
pub fn filter_by_search<'a>(downloads: &[&'a Download], query: &str) -> Vec<&'a Download> {
    if query.is_empty() {
        return downloads.to_vec();
    }
    let query_lower = query.to_lowercase();
    downloads
        .iter()
        .filter(|d| d.name.to_lowercase().contains(&query_lower))
        .copied()
        .collect()
}

/// Sort order for downloads
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortOrder {
    Name,
    Size,
    Progress,
    Speed,
    Status,
}

impl SortOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortOrder::Name => "Name",
            SortOrder::Size => "Size",
            SortOrder::Progress => "Progress",
            SortOrder::Speed => "Speed",
            SortOrder::Status => "Status",
        }
    }

    pub fn next(&self) -> SortOrder {
        match self {
            SortOrder::Name => SortOrder::Size,
            SortOrder::Size => SortOrder::Progress,
            SortOrder::Progress => SortOrder::Speed,
            SortOrder::Speed => SortOrder::Status,
            SortOrder::Status => SortOrder::Name,
        }
    }

    #[allow(dead_code)]
    pub fn prev(&self) -> SortOrder {
        match self {
            SortOrder::Name => SortOrder::Status,
            SortOrder::Size => SortOrder::Name,
            SortOrder::Progress => SortOrder::Size,
            SortOrder::Speed => SortOrder::Progress,
            SortOrder::Status => SortOrder::Speed,
        }
    }
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Name
    }
}

/// Sort downloads by the given order
pub fn sort_downloads<'a>(downloads: &mut [&'a Download], order: SortOrder, ascending: bool) {
    downloads.sort_by(|a, b| {
        let cmp = match order {
            SortOrder::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortOrder::Size => a.total_length.cmp(&b.total_length),
            SortOrder::Progress => a
                .progress
                .partial_cmp(&b.progress)
                .unwrap_or(std::cmp::Ordering::Equal),
            SortOrder::Speed => {
                let speed_a = parse_speed(&a.speed);
                let speed_b = parse_speed(&b.speed);
                speed_a.cmp(&speed_b)
            }
            SortOrder::Status => a.status.cmp(&b.status),
        };
        if ascending {
            cmp
        } else {
            cmp.reverse()
        }
    });
}

/// Parse speed string back to bytes/sec for comparison
fn parse_speed(speed: &str) -> u64 {
    let parts: Vec<&str> = speed.split_whitespace().collect();
    if parts.len() < 2 {
        return 0;
    }

    let value: f64 = parts[0].parse().unwrap_or(0.0);
    let unit = parts[1].to_uppercase();

    match unit.as_str() {
        "B/S" => value as u64,
        "KB/S" => (value * 1024.0) as u64,
        "MB/S" => (value * 1024.0 * 1024.0) as u64,
        "GB/S" => (value * 1024.0 * 1024.0 * 1024.0) as u64,
        _ => 0,
    }
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

/// Check if a download has an error
pub fn is_error(download: &Download) -> bool {
    download.status == "ERROR" || download.status.to_lowercase().contains("error")
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

/// Format ETA (Estimated Time of Arrival) from remaining bytes and speed
pub fn format_eta(remaining_bytes: u64, speed_bytes_per_sec: u64) -> String {
    if speed_bytes_per_sec == 0 {
        return "∞".to_string();
    }

    let seconds = remaining_bytes / speed_bytes_per_sec;
    format_duration(seconds)
}

/// Format ETA for a download based on its current state
pub fn format_download_eta(download: &Download) -> String {
    if download.progress >= 1.0 {
        return "Complete".to_string();
    }

    if download.status == "PAUSED" {
        return "Paused".to_string();
    }

    if download.status == "WAITING" {
        return "Waiting".to_string();
    }

    if download.status == "ERROR" || download.status.to_lowercase().contains("error") {
        return "Error".to_string();
    }

    // Get average speed from history for more stable ETA
    let avg_speed = if !download.speed_history.is_empty() {
        let sum: u64 = download.speed_history.iter().sum();
        sum / download.speed_history.len() as u64
    } else {
        // Parse current speed string
        parse_speed(&download.speed)
    };

    if avg_speed == 0 {
        return "∞".to_string();
    }

    let remaining = download
        .total_length
        .saturating_sub(download.completed_length);
    format_eta(remaining, avg_speed)
}

/// Format duration in human-readable format
pub fn format_duration(total_seconds: u64) -> String {
    if total_seconds == 0 {
        return "0s".to_string();
    }

    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
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

/// Global statistics for all downloads
#[derive(Clone, Debug, Default)]
pub struct GlobalStats {
    pub total_download_speed: u64,
    pub total_upload_speed: u64,
    pub active_count: usize,
    pub waiting_count: usize,
    pub completed_count: usize,
    pub error_count: usize,
    pub total_downloaded: u64,
    pub total_size: u64,
}

/// Calculate global statistics from all downloads
pub fn calculate_global_stats(downloads: &[Download]) -> GlobalStats {
    let mut stats = GlobalStats::default();

    for download in downloads {
        // Count by status
        if is_active(download) {
            stats.active_count += 1;
            // Sum speeds for active downloads
            stats.total_download_speed += parse_speed(&download.speed);
            stats.total_upload_speed += parse_speed(&download.upload_speed);
        } else if is_completed(download) {
            stats.completed_count += 1;
        } else if is_error(download) {
            stats.error_count += 1;
        } else if is_queued(download) {
            stats.waiting_count += 1;
        }

        // Sum sizes
        stats.total_downloaded += download.completed_length;
        stats.total_size += download.total_length;
    }

    stats
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
            progress,
            speed: "0 B/s".to_string(),
            status: status.to_string(),
            total_length: 0,
            completed_length: 0,
            download_type: DownloadType::Http,
            speed_history: vec![],
            connections: 0,
            file_path: None,
            error_message: None,
            upload_speed: "".to_string(),
            upload_speed_history: vec![0, 0],
            added_at: std::time::Instant::now(),
            seeds: 0,
            peers: 0,
            bitfield: None,
            num_pieces: 0,
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
    fn test_format_eta() {
        assert_eq!(format_eta(1024, 0), "∞");
        assert_eq!(format_eta(60, 1), "1m 0s");
        assert_eq!(format_eta(3600, 1), "1h 0m");
        assert_eq!(format_eta(86400, 1), "1d 0h");
        assert_eq!(format_eta(1024, 1024), "1s");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m");
        assert_eq!(format_duration(90061), "1d 1h");
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

    #[test]
    fn test_is_error() {
        let download = create_test_download("ERROR", 0.0);
        assert!(is_error(&download));

        let download = create_test_download("ACTIVE", 0.5);
        assert!(!is_error(&download));
    }

    #[test]
    fn test_filter_by_search() {
        let d1 = create_test_download("ACTIVE", 0.5);
        let mut d2 = create_test_download("ACTIVE", 0.3);
        d2.name = "other_file.zip".to_string();

        let downloads: Vec<&Download> = vec![&d1, &d2];

        let filtered = filter_by_search(&downloads, "test");
        assert_eq!(filtered.len(), 1);

        let filtered = filter_by_search(&downloads, "");
        assert_eq!(filtered.len(), 2);

        let filtered = filter_by_search(&downloads, "TEST");
        assert_eq!(filtered.len(), 1); // Case insensitive
    }

    #[test]
    fn test_sort_order() {
        assert_eq!(SortOrder::Name.next(), SortOrder::Size);
        assert_eq!(SortOrder::Status.next(), SortOrder::Name);
        assert_eq!(SortOrder::Name.prev(), SortOrder::Status);
    }

    #[test]
    fn test_parse_speed() {
        assert_eq!(parse_speed("100 B/s"), 100);
        assert_eq!(parse_speed("1.00 KB/s"), 1024);
        assert_eq!(parse_speed("1.00 MB/s"), 1048576);
        assert_eq!(parse_speed("invalid"), 0);
    }

    #[test]
    fn test_global_stats() {
        let d1 = create_test_download("ACTIVE", 0.5);
        let d2 = create_test_download("COMPLETE", 1.0);
        let d3 = create_test_download("WAITING", 0.0);

        let downloads = vec![d1, d2, d3];
        let stats = calculate_global_stats(&downloads);

        assert_eq!(stats.active_count, 1);
        assert_eq!(stats.completed_count, 1);
        assert_eq!(stats.waiting_count, 1);
    }
}
