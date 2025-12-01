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
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Normal
    }
}
