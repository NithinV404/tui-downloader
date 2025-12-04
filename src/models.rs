/// Represents a download item with metadata
#[derive(Clone, Debug)]
pub struct Download {
    pub gid: Option<String>,
    pub name: String,
    pub url: Option<String>,
    pub progress: f64,
    pub speed: String,
    pub status: String,
    pub total_length: u64,
    pub completed_length: u64,
    pub download_type: DownloadType,
    pub speed_history: Vec<u64>, // Download speed in bytes/sec for graphing
    pub upload_speed: String,
    pub upload_speed_history: Vec<u64>, // Upload speed in bytes/sec for graphing
    pub connections: u32,
    pub file_path: Option<String>,
    pub error_message: Option<String>,
    #[allow(dead_code)]
    pub added_at: std::time::Instant, // When the download was added
    pub seeds: u32,               // For torrents: number of seeders
    pub peers: u32,               // For torrents: number of peers
    pub bitfield: Option<String>, // Hex string showing which pieces are downloaded
    pub num_pieces: u32,          // Total number of pieces in the download
}

impl Default for Download {
    fn default() -> Self {
        Self {
            gid: None,
            name: String::new(),
            url: None,
            progress: 0.0,
            speed: "0 B/s".to_string(),
            status: "IDLE".to_string(),
            total_length: 0,
            completed_length: 0,
            download_type: DownloadType::Http,
            speed_history: Vec::new(),
            upload_speed: "0 B/s".to_string(),
            upload_speed_history: Vec::new(),
            connections: 0,
            file_path: None,
            error_message: None,
            added_at: std::time::Instant::now(),
            seeds: 0,
            peers: 0,
            bitfield: None,
            num_pieces: 0,
        }
    }
}

/// Type of download
#[derive(Clone, Debug, PartialEq)]
pub enum DownloadType {
    Http,
    Torrent,
    Metalink,
}

/// Input mode for the application
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum InputMode {
    Normal,
    Editing,
    Search,
    SpeedLimit,
    Help,
    Confirmation,
    #[allow(dead_code)]
    Settings,
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Normal
    }
}

/// Global statistics from aria2
#[derive(Clone, Debug, Default)]
pub struct GlobalStats {
    pub download_speed: u64,
    pub upload_speed: u64,
    pub num_active: u32,
    pub num_waiting: u32,
    pub num_stopped: u32,
    pub num_stopped_total: u32,
}

/// Sorting options for downloads
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum SortField {
    #[default]
    Name,
    Size,
    Progress,
    Speed,
    DateAdded,
    Status,
}

#[allow(dead_code)]
impl SortField {
    pub fn next(&self) -> Self {
        match self {
            SortField::Name => SortField::Size,
            SortField::Size => SortField::Progress,
            SortField::Progress => SortField::Speed,
            SortField::Speed => SortField::DateAdded,
            SortField::DateAdded => SortField::Status,
            SortField::Status => SortField::Name,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SortField::Name => "Name",
            SortField::Size => "Size",
            SortField::Progress => "Progress",
            SortField::Speed => "Speed",
            SortField::DateAdded => "Date Added",
            SortField::Status => "Status",
        }
    }
}

/// Sort direction
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

#[allow(dead_code)]
impl SortDirection {
    pub fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SortDirection::Ascending => "^",
            SortDirection::Descending => "v",
        }
    }
}

/// Confirmation action type
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum ConfirmAction {
    Quit,
    DeleteFile(String), // GID of download to delete
    PurgeCompleted,
    RetryDownload(String), // GID of download to retry
}

/// Speed limit settings
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct SpeedLimitSettings {
    pub download_limit: u64,    // bytes per second, 0 = unlimited
    pub upload_limit: u64,      // bytes per second, 0 = unlimited
    pub editing_download: bool, // true = editing download, false = editing upload
}

#[allow(dead_code)]
impl SpeedLimitSettings {
    pub fn format_limit(limit: u64) -> String {
        if limit == 0 {
            "Unlimited".to_string()
        } else {
            crate::ui::utils::format_speed(limit)
        }
    }

    pub fn parse_limit(input: &str) -> Option<u64> {
        let input = input.trim().to_lowercase();

        if input == "0" || input.is_empty() || input == "unlimited" {
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
        } else {
            1 // bytes
        };

        Some((num * multiplier as f64) as u64)
    }
}

/// Application settings
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct AppSettings {
    pub download_dir: String,
    pub max_connections: u32,
    pub max_concurrent_downloads: u32,
    pub split_size: String,
    pub seed_time: u32,
    pub theme: String,
}

#[allow(dead_code)]
impl Default for AppSettings {
    fn default() -> Self {
        let download_dir = dirs::download_dir()
            .or_else(|| dirs::home_dir().map(|p| p.join("Downloads")))
            .unwrap_or_else(|| std::path::PathBuf::from("./Downloads"))
            .display()
            .to_string();

        Self {
            download_dir,
            max_connections: 16,
            max_concurrent_downloads: 5,
            split_size: "1M".to_string(),
            seed_time: 0,
            theme: "dark".to_string(),
        }
    }
}

/// URL history for autocomplete
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct UrlHistory {
    pub entries: Vec<String>,
    pub max_entries: usize,
}

#[allow(dead_code)]
impl UrlHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    pub fn add(&mut self, url: &str) {
        // Remove duplicates
        self.entries.retain(|u| u != url);

        // Add to front
        self.entries.insert(0, url.to_string());

        // Trim to max size
        if self.entries.len() > self.max_entries {
            self.entries.truncate(self.max_entries);
        }
    }

    pub fn filter(&self, prefix: &str) -> Vec<&str> {
        self.entries
            .iter()
            .filter(|u| u.to_lowercase().contains(&prefix.to_lowercase()))
            .map(|s| s.as_str())
            .take(5)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_field_cycle() {
        let mut field = SortField::Name;
        field = field.next();
        assert_eq!(field, SortField::Size);
        field = field.next();
        assert_eq!(field, SortField::Progress);
    }

    #[test]
    fn test_sort_direction_toggle() {
        let dir = SortDirection::Ascending;
        assert_eq!(dir.toggle(), SortDirection::Descending);
        assert_eq!(dir.toggle().toggle(), SortDirection::Ascending);
    }

    #[test]
    fn test_speed_limit_parse() {
        assert_eq!(SpeedLimitSettings::parse_limit("5m"), Some(5 * 1024 * 1024));
        assert_eq!(
            SpeedLimitSettings::parse_limit("5 MB/s"),
            Some(5 * 1024 * 1024)
        );
        assert_eq!(SpeedLimitSettings::parse_limit("500k"), Some(500 * 1024));
        assert_eq!(SpeedLimitSettings::parse_limit("0"), Some(0));
        assert_eq!(SpeedLimitSettings::parse_limit("unlimited"), Some(0));
    }

    #[test]
    fn test_url_history() {
        let mut history = UrlHistory::new(3);
        history.add("https://example.com/file1.zip");
        history.add("https://example.com/file2.zip");
        history.add("https://example.com/file3.zip");
        history.add("https://example.com/file4.zip");

        assert_eq!(history.entries.len(), 3);
        assert_eq!(history.entries[0], "https://example.com/file4.zip");
    }

    #[test]
    fn test_url_history_filter() {
        let mut history = UrlHistory::new(10);
        history.add("https://example.com/video.mp4");
        history.add("https://test.com/audio.mp3");
        history.add("https://example.com/image.png");

        let filtered = history.filter("example");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_download_default() {
        let download = Download::default();
        assert_eq!(download.status, "IDLE");
        assert_eq!(download.progress, 0.0);
    }
}
