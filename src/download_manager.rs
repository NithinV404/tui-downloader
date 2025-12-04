use crate::aria2::{Aria2Manager, Aria2Status};
use crate::models::{Download, DownloadType, GlobalStats};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

const MAX_SPEED_HISTORY: usize = 60; // Keep last 60 data points for graphing

pub struct DownloadManager {
    aria2: Arc<Aria2Manager>,
    downloads: Arc<RwLock<HashMap<String, Download>>>,
    deleted_gids: Arc<RwLock<HashSet<String>>>, // Track deleted GIDs to prevent re-adding
    global_stats: Arc<RwLock<GlobalStats>>,
}

#[allow(dead_code)]
impl DownloadManager {
    /// Create a new download manager
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let aria2 = Arc::new(Aria2Manager::new().await?);
        let downloads = Arc::new(RwLock::new(HashMap::new()));
        let deleted_gids = Arc::new(RwLock::new(HashSet::new()));
        let global_stats = Arc::new(RwLock::new(GlobalStats::default()));

        Ok(Self {
            aria2,
            downloads,
            deleted_gids,
            global_stats,
        })
    }

    /// Add a download from URL, torrent file, or magnet link
    pub async fn add_download(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let gid = if input.starts_with("magnet:") {
            // Magnet link - treat as torrent
            self.aria2.add_uri(input).await?
        } else if input.ends_with(".torrent") {
            // Torrent file path
            self.aria2.add_torrent(input).await?
        } else if input.ends_with(".metalink") || input.ends_with(".meta4") {
            // Metalink file
            self.aria2.add_metalink(input).await?
        } else {
            // Regular HTTP/HTTPS/FTP URL
            self.aria2.add_uri(input).await?
        };

        // Create initial download entry
        let download_type = if input.starts_with("magnet:") || input.ends_with(".torrent") {
            DownloadType::Torrent
        } else if input.ends_with(".metalink") || input.ends_with(".meta4") {
            DownloadType::Metalink
        } else {
            DownloadType::Http
        };

        let download = Download {
            gid: Some(gid.clone()),
            name: extract_filename(input),
            url: Some(input.to_string()),
            progress: 0.0,
            speed: "0 B/s".to_string(),
            status: "WAITING".to_string(),
            total_length: 0,
            completed_length: 0,
            download_type,
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
        };

        self.downloads.write().await.insert(gid.clone(), download);

        Ok(gid)
    }

    /// Retry a failed download by re-adding it
    pub async fn retry_download(&self, gid: &str) -> Result<String, Box<dyn std::error::Error>> {
        let download = self.downloads.read().await.get(gid).cloned();

        if let Some(download) = download {
            if let Some(url) = &download.url {
                // Remove the old download
                self.remove_download(gid).await?;

                // Add it again
                let new_gid = self.add_download(url).await?;
                Ok(new_gid)
            } else {
                Err("No URL available for retry".into())
            }
        } else {
            Err("Download not found".into())
        }
    }

    /// Update download information from aria2c
    pub async fn update_downloads(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Get all active downloads
        let active = self.aria2.get_active().await.unwrap_or_default();

        // Get waiting downloads
        let waiting = self.aria2.get_waiting(0, 100).await.unwrap_or_default();

        // Get stopped downloads (completed or error)
        let stopped = self.aria2.get_stopped(0, 100).await.unwrap_or_default();

        let mut downloads = self.downloads.write().await;
        let deleted_gids = self.deleted_gids.read().await;

        // Update active downloads
        for status in active {
            if !deleted_gids.contains(&status.gid) {
                self.update_download_from_status(&mut downloads, status, &deleted_gids)
                    .await;
            }
        }

        // Update waiting downloads
        for status in waiting {
            if !deleted_gids.contains(&status.gid) {
                self.update_download_from_status(&mut downloads, status, &deleted_gids)
                    .await;
            }
        }

        // Update stopped downloads
        for status in stopped {
            if !deleted_gids.contains(&status.gid) {
                self.update_download_from_status(&mut downloads, status, &deleted_gids)
                    .await;
            }
        }

        // Update global stats
        self.update_global_stats(&downloads).await;

        Ok(())
    }

    async fn update_global_stats(&self, downloads: &HashMap<String, Download>) {
        let mut stats = GlobalStats::default();

        for download in downloads.values() {
            match download.status.as_str() {
                "ACTIVE" => {
                    stats.num_active += 1;
                    // Parse speed strings to get total speeds
                    if let Some(speed) = download.speed_history.last() {
                        stats.download_speed += speed;
                    }
                    if let Some(speed) = download.upload_speed_history.last() {
                        stats.upload_speed += speed;
                    }
                }
                "WAITING" | "PAUSED" => stats.num_waiting += 1,
                "COMPLETE" => stats.num_stopped += 1,
                "ERROR" => stats.num_stopped += 1,
                _ => {}
            }
        }

        stats.num_stopped_total = downloads.len() as u32;
        *self.global_stats.write().await = stats;
    }

    async fn update_download_from_status(
        &self,
        downloads: &mut HashMap<String, Download>,
        status: Aria2Status,
        deleted_gids: &tokio::sync::RwLockReadGuard<'_, HashSet<String>>,
    ) {
        // Skip if this download was deleted by user
        if deleted_gids.contains(&status.gid) {
            return;
        }

        if let Some(download) = downloads.get_mut(&status.gid) {
            // Update existing download
            let total: u64 = status.total_length.parse().unwrap_or(0);
            let completed: u64 = status.completed_length.parse().unwrap_or(0);
            let speed: u64 = status.download_speed.parse().unwrap_or(0);
            let upload_speed: u64 = status.upload_speed.parse().unwrap_or(0);

            download.progress = if total > 0 {
                completed as f64 / total as f64
            } else {
                0.0
            };
            download.speed = format_speed(speed);
            download.upload_speed = format_speed(upload_speed);
            download.status = status.status.to_uppercase();
            download.total_length = total;
            download.completed_length = completed;
            download.connections = status.connections.parse().unwrap_or(0);

            // Update error message if present
            download.error_message = status.error_message.clone();

            // Update seeds and peers from bittorrent info if available
            if let Some(ref bt_info) = status.bittorrent {
                download.seeds = bt_info.num_seeders.parse().unwrap_or(0);
            }
            download.peers = status
                .num_peers
                .as_ref()
                .and_then(|p| p.parse().ok())
                .unwrap_or(0);

            // Update bitfield and piece count for piece visualization
            download.bitfield = status.bitfield.clone();
            download.num_pieces = status
                .num_pieces
                .as_ref()
                .and_then(|p| p.parse().ok())
                .unwrap_or(0);

            // Update download speed history for graphing
            download.speed_history.push(speed);
            if download.speed_history.len() > MAX_SPEED_HISTORY {
                download.speed_history.remove(0);
            }

            // Update upload speed history for graphing
            download.upload_speed_history.push(upload_speed);
            if download.upload_speed_history.len() > MAX_SPEED_HISTORY {
                download.upload_speed_history.remove(0);
            }

            // Extract filename and path from aria2 files if available
            if let Some(files) = &status.files {
                if let Some(file) = files.first() {
                    if !file.path.is_empty() {
                        download.file_path = Some(file.path.clone());
                        // Extract just the filename from the path
                        if let Some(filename) = file.path.split('/').last() {
                            if !filename.is_empty() {
                                download.name = filename.to_string();
                            }
                        }
                    }
                }
            }
        } else {
            // New download not added by us - add it
            let total: u64 = status.total_length.parse().unwrap_or(0);
            let completed: u64 = status.completed_length.parse().unwrap_or(0);
            let speed: u64 = status.download_speed.parse().unwrap_or(0);
            let upload_speed: u64 = status.upload_speed.parse().unwrap_or(0);

            let mut name = "Unknown".to_string();
            let mut file_path = None;

            // Try to extract filename from files
            if let Some(files) = &status.files {
                if let Some(file) = files.first() {
                    if !file.path.is_empty() {
                        file_path = Some(file.path.clone());
                        if let Some(filename) = file.path.split('/').last() {
                            if !filename.is_empty() {
                                name = filename.to_string();
                            }
                        }
                    }
                }
            }

            // Determine download type from bittorrent info
            let download_type = if status.bittorrent.is_some() {
                DownloadType::Torrent
            } else {
                DownloadType::Http
            };

            let seeds = status
                .bittorrent
                .as_ref()
                .map(|bt| bt.num_seeders.parse().unwrap_or(0))
                .unwrap_or(0);
            let peers = status
                .num_peers
                .as_ref()
                .and_then(|p| p.parse().ok())
                .unwrap_or(0);
            let num_pieces = status
                .num_pieces
                .as_ref()
                .and_then(|p| p.parse().ok())
                .unwrap_or(0);

            let download = Download {
                gid: Some(status.gid.clone()),
                name,
                url: None,
                progress: if total > 0 {
                    completed as f64 / total as f64
                } else {
                    0.0
                },
                speed: format_speed(speed),
                status: status.status.to_uppercase(),
                total_length: total,
                completed_length: completed,
                download_type,
                speed_history: vec![speed],
                upload_speed: format_speed(upload_speed),
                upload_speed_history: vec![upload_speed],
                connections: status.connections.parse().unwrap_or(0),
                file_path,
                error_message: status.error_message.clone(),
                added_at: std::time::Instant::now(),
                seeds,
                peers,
                bitfield: status.bitfield.clone(),
                num_pieces,
            };

            downloads.insert(status.gid, download);
        }
    }

    /// Get all downloads
    pub async fn get_all_downloads(&self) -> Vec<Download> {
        self.downloads.read().await.values().cloned().collect()
    }

    /// Get active downloads
    pub async fn get_active_downloads(&self) -> Vec<Download> {
        self.downloads
            .read()
            .await
            .values()
            .filter(|d| d.progress > 0.0 && d.progress < 1.0)
            .cloned()
            .collect()
    }

    /// Get queued downloads
    pub async fn get_queued_downloads(&self) -> Vec<Download> {
        self.downloads
            .read()
            .await
            .values()
            .filter(|d| d.status == "WAITING" || d.progress == 0.0)
            .cloned()
            .collect()
    }

    /// Get completed downloads
    pub async fn get_completed_downloads(&self) -> Vec<Download> {
        self.downloads
            .read()
            .await
            .values()
            .filter(|d| d.progress >= 1.0 || d.status == "COMPLETE")
            .cloned()
            .collect()
    }

    /// Get global statistics
    pub async fn get_global_stats(&self) -> GlobalStats {
        self.global_stats.read().await.clone()
    }

    /// Pause a download
    pub async fn pause_download(&self, gid: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.aria2.pause(gid).await?;
        Ok(())
    }

    /// Resume a download
    pub async fn resume_download(&self, gid: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.aria2.unpause(gid).await?;
        Ok(())
    }

    /// Remove a download
    pub async fn remove_download(&self, gid: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Mark as deleted FIRST to prevent re-adding during async operations
        self.deleted_gids.write().await.insert(gid.to_string());

        // Remove from our local storage
        self.downloads.write().await.remove(gid);

        // Try to force remove from aria2c (might fail if already stopped)
        let _ = self.aria2.force_remove(gid).await;

        // Small delay to ensure aria2c processes the removal
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Remove from aria2c's completed/stopped list
        let _ = self.aria2.remove_download_result(gid).await;

        Ok(())
    }

    /// Get download by GID
    pub async fn get_download(&self, gid: &str) -> Option<Download> {
        self.downloads.read().await.get(gid).cloned()
    }

    /// Purge all completed downloads
    pub async fn purge_completed(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let completed_gids: Vec<String> = self
            .downloads
            .read()
            .await
            .iter()
            .filter(|(_, d)| d.progress >= 1.0 || d.status == "COMPLETE")
            .map(|(gid, _)| gid.clone())
            .collect();

        let count = completed_gids.len();

        for gid in completed_gids {
            self.remove_download(&gid).await?;
        }

        // Also purge from aria2c
        let _ = self.aria2.purge_download_result().await;

        Ok(count)
    }

    /// Delete downloaded file from disk and remove from download list
    pub async fn delete_file(&self, gid: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Get the download to find the file path
        let download = self.downloads.read().await.get(gid).cloned();

        if let Some(download) = download {
            let file_path = download.file_path.clone();
            let file_name = download.name.clone();

            // Remove from download list first
            self.remove_download(gid).await?;

            // Delete the actual file if it exists
            if let Some(path) = file_path {
                match tokio::fs::remove_file(&path).await {
                    Ok(_) => Ok(format!("Deleted file: {}", file_name)),
                    Err(e) => {
                        // File might not exist or we don't have permission
                        Err(format!("Failed to delete file {}: {}", file_name, e).into())
                    }
                }
            } else {
                Ok(format!("Removed from list: {} (file not found)", file_name))
            }
        } else {
            Err("Download not found".into())
        }
    }

    /// Set global download speed limit (0 = unlimited)
    pub async fn set_download_speed_limit(
        &self,
        limit: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.aria2
            .set_global_option("max-overall-download-limit", &format!("{}", limit))
            .await?;
        Ok(())
    }

    /// Set global upload speed limit (0 = unlimited)
    pub async fn set_upload_speed_limit(
        &self,
        limit: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.aria2
            .set_global_option("max-overall-upload-limit", &format!("{}", limit))
            .await?;
        Ok(())
    }

    /// Get current speed limits
    pub async fn get_speed_limits(&self) -> Result<(u64, u64), Box<dyn std::error::Error>> {
        let options = self.aria2.get_global_option().await?;

        let download_limit = options
            .get("max-overall-download-limit")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let upload_limit = options
            .get("max-overall-upload-limit")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        Ok((download_limit, upload_limit))
    }

    /// Move download up in queue
    pub async fn move_up(&self, gid: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.aria2.change_position(gid, -1, "POS_CUR").await?;
        Ok(())
    }

    /// Move download down in queue
    pub async fn move_down(&self, gid: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.aria2.change_position(gid, 1, "POS_CUR").await?;
        Ok(())
    }

    /// Pause all downloads
    pub async fn pause_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.aria2.pause_all().await?;
        Ok(())
    }

    /// Resume all downloads
    pub async fn resume_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.aria2.unpause_all().await?;
        Ok(())
    }

    /// Shutdown aria2c
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.aria2.shutdown().await
    }
}

/// Extract filename from URL or path
fn extract_filename(input: &str) -> String {
    if input.starts_with("magnet:") {
        // Extract name from magnet link
        if let Some(dn_start) = input.find("dn=") {
            let name_part = &input[dn_start + 3..];
            if let Some(end) = name_part.find('&') {
                return urlencoding::decode(&name_part[..end])
                    .unwrap_or_default()
                    .to_string();
            } else {
                return urlencoding::decode(name_part)
                    .unwrap_or_default()
                    .to_string();
            }
        }
        return "Magnet Download".to_string();
    }

    // For URLs and file paths
    let path = input.split('?').next().unwrap_or(input);
    let filename = path.split('/').last().unwrap_or("Unknown");

    if filename.is_empty() {
        "Unknown".to_string()
    } else {
        filename.to_string()
    }
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
        format!("{:.2} KB/s", speed_bytes as f64 / KB as f64)
    } else {
        format!("{} B/s", speed_bytes)
    }
}

/// Format file size in human-readable format

mod urlencoding {
    pub fn decode(s: &str) -> Result<String, ()> {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '%' {
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte as char);
                    } else {
                        result.push('%');
                        result.push_str(&hex);
                    }
                } else {
                    result.push('%');
                    result.push_str(&hex);
                }
            } else if c == '+' {
                result.push(' ');
            } else {
                result.push(c);
            }
        }

        Ok(result)
    }
}
