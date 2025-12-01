use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;

const ARIA2C_RPC_PORT: u16 = 6800;
const ARIA2C_RPC_SECRET: &str = "tui_downloader_secret";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aria2Status {
    pub gid: String,
    pub status: String,
    #[serde(rename = "totalLength")]
    pub total_length: String,
    #[serde(rename = "completedLength")]
    pub completed_length: String,
    #[serde(rename = "downloadSpeed")]
    pub download_speed: String,
    #[serde(rename = "uploadSpeed")]
    pub upload_speed: String,
    pub connections: String,
    #[serde(rename = "errorCode")]
    pub error_code: Option<String>,
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
    pub files: Option<Vec<Aria2File>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aria2File {
    pub index: String,
    pub path: String,
    pub length: String,
    #[serde(rename = "completedLength")]
    pub completed_length: String,
    pub selected: String,
    pub uris: Option<Vec<FileUri>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUri {
    pub uri: String,
    pub status: String,
}

pub struct Aria2Manager {
    process: Arc<Mutex<Option<Child>>>,
    rpc_url: String,
    secret: String,
    client: reqwest::Client,
}

#[allow(dead_code)]
impl Aria2Manager {
    /// Creates a new Aria2Manager and automatically spawns aria2c if not running
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let rpc_url = format!("http://localhost:{}/jsonrpc", ARIA2C_RPC_PORT);
        let secret = ARIA2C_RPC_SECRET.to_string();

        let manager = Self {
            process: Arc::new(Mutex::new(None)),
            rpc_url,
            secret,
            client,
        };

        // Try to connect to existing aria2c instance
        if !manager.is_running().await {
            manager.spawn_aria2c().await?;

            // Wait for aria2c to start
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            if !manager.is_running().await {
                return Err("Failed to start aria2c".into());
            }
        }

        Ok(manager)
    }

    /// Spawns aria2c process with proper configuration
    async fn spawn_aria2c(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Get or create Downloads directory
        let download_dir = dirs::download_dir()
            .or_else(|| dirs::home_dir().map(|p| p.join("Downloads")))
            .unwrap_or_else(|| std::path::PathBuf::from("./Downloads"));

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&download_dir)?;

        let child = Command::new("aria2c")
            .args(&[
                "--enable-rpc",
                "--rpc-listen-all=false",
                &format!("--rpc-listen-port={}", ARIA2C_RPC_PORT),
                &format!("--rpc-secret={}", ARIA2C_RPC_SECRET),
                &format!("--dir={}", download_dir.display()),
                "--continue=true",
                "--max-connection-per-server=16",
                "--min-split-size=1M",
                "--split=16",
                "--max-concurrent-downloads=5",
                "--disable-ipv6=false",
                "--seed-time=0", // Don't seed torrents after download
                "--bt-max-peers=50",
                "--follow-torrent=true",
                "--enable-dht=true",
                "--bt-enable-lpd=true",
                "--enable-peer-exchange=true",
                "--auto-file-renaming=false",
                "--allow-overwrite=true",
                "--summary-interval=0",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        *self.process.lock().await = Some(child);
        Ok(())
    }

    /// Check if aria2c is running by making a test RPC call
    async fn is_running(&self) -> bool {
        self.call_method("aria2.getVersion", vec![]).await.is_ok()
    }

    /// Make a JSON-RPC call to aria2c
    async fn call_method(
        &self,
        method: &str,
        params: Vec<Value>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let mut rpc_params = vec![json!(format!("token:{}", self.secret))];
        rpc_params.extend(params);

        let payload = json!({
            "jsonrpc": "2.0",
            "id": uuid::Uuid::new_v4().to_string(),
            "method": method,
            "params": rpc_params,
        });

        let response = self
            .client
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .await?;

        let result: Value = response.json().await?;

        if let Some(error) = result.get("error") {
            return Err(format!("Aria2 RPC error: {}", error).into());
        }

        Ok(result["result"].clone())
    }

    /// Add a URL download
    pub async fn add_uri(&self, uri: &str) -> Result<String, Box<dyn std::error::Error>> {
        let uris = vec![json!(uri)];
        let result = self.call_method("aria2.addUri", vec![json!(uris)]).await?;

        Ok(result.as_str().unwrap_or("").to_string())
    }

    /// Add a torrent file
    pub async fn add_torrent(
        &self,
        torrent_path: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Read torrent file and encode as base64
        let torrent_data = tokio::fs::read(torrent_path).await?;
        let encoded = base64::encode(&torrent_data);

        let result = self
            .call_method("aria2.addTorrent", vec![json!(encoded)])
            .await?;

        Ok(result.as_str().unwrap_or("").to_string())
    }

    /// Add a metalink file
    pub async fn add_metalink(
        &self,
        metalink_path: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let metalink_data = tokio::fs::read(metalink_path).await?;
        let encoded = base64::encode(&metalink_data);

        let result = self
            .call_method("aria2.addMetalink", vec![json!(encoded)])
            .await?;

        Ok(result.as_str().unwrap_or("").to_string())
    }

    /// Get status of a download by GID with file information
    pub async fn get_status(&self, gid: &str) -> Result<Aria2Status, Box<dyn std::error::Error>> {
        let result = self
            .call_method(
                "aria2.tellStatus",
                vec![
                    json!(gid),
                    json!([
                        "gid",
                        "status",
                        "totalLength",
                        "completedLength",
                        "downloadSpeed",
                        "uploadSpeed",
                        "connections",
                        "errorCode",
                        "errorMessage",
                        "files"
                    ]),
                ],
            )
            .await?;

        Ok(serde_json::from_value(result)?)
    }

    /// Get files for a download
    pub async fn get_files(&self, gid: &str) -> Result<Vec<Aria2File>, Box<dyn std::error::Error>> {
        let result = self.call_method("aria2.getFiles", vec![json!(gid)]).await?;

        Ok(serde_json::from_value(result)?)
    }

    /// Get all active downloads
    pub async fn get_active(&self) -> Result<Vec<Aria2Status>, Box<dyn std::error::Error>> {
        let result = self
            .call_method(
                "aria2.tellActive",
                vec![json!([
                    "gid",
                    "status",
                    "totalLength",
                    "completedLength",
                    "downloadSpeed",
                    "uploadSpeed",
                    "connections",
                    "errorCode",
                    "errorMessage",
                    "files"
                ])],
            )
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get waiting downloads
    pub async fn get_waiting(
        &self,
        offset: i32,
        num: i32,
    ) -> Result<Vec<Aria2Status>, Box<dyn std::error::Error>> {
        let result = self
            .call_method(
                "aria2.tellWaiting",
                vec![
                    json!(offset),
                    json!(num),
                    json!([
                        "gid",
                        "status",
                        "totalLength",
                        "completedLength",
                        "downloadSpeed",
                        "uploadSpeed",
                        "connections",
                        "errorCode",
                        "errorMessage",
                        "files"
                    ]),
                ],
            )
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get stopped downloads
    pub async fn get_stopped(
        &self,
        offset: i32,
        num: i32,
    ) -> Result<Vec<Aria2Status>, Box<dyn std::error::Error>> {
        let result = self
            .call_method(
                "aria2.tellStopped",
                vec![
                    json!(offset),
                    json!(num),
                    json!([
                        "gid",
                        "status",
                        "totalLength",
                        "completedLength",
                        "downloadSpeed",
                        "uploadSpeed",
                        "connections",
                        "errorCode",
                        "errorMessage",
                        "files"
                    ]),
                ],
            )
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Pause a download
    pub async fn pause(&self, gid: &str) -> Result<String, Box<dyn std::error::Error>> {
        let result = self.call_method("aria2.pause", vec![json!(gid)]).await?;
        Ok(result.as_str().unwrap_or("").to_string())
    }

    /// Pause all downloads
    pub async fn pause_all(&self) -> Result<String, Box<dyn std::error::Error>> {
        let result = self.call_method("aria2.pauseAll", vec![]).await?;
        Ok(result.as_str().unwrap_or("OK").to_string())
    }

    /// Unpause a download
    pub async fn unpause(&self, gid: &str) -> Result<String, Box<dyn std::error::Error>> {
        let result = self.call_method("aria2.unpause", vec![json!(gid)]).await?;
        Ok(result.as_str().unwrap_or("").to_string())
    }

    /// Unpause all downloads
    pub async fn unpause_all(&self) -> Result<String, Box<dyn std::error::Error>> {
        let result = self.call_method("aria2.unpauseAll", vec![]).await?;
        Ok(result.as_str().unwrap_or("OK").to_string())
    }

    /// Remove a download
    pub async fn remove(&self, gid: &str) -> Result<String, Box<dyn std::error::Error>> {
        let result = self.call_method("aria2.remove", vec![json!(gid)]).await?;
        Ok(result.as_str().unwrap_or("").to_string())
    }

    /// Force remove a download
    pub async fn force_remove(&self, gid: &str) -> Result<String, Box<dyn std::error::Error>> {
        let result = self
            .call_method("aria2.forceRemove", vec![json!(gid)])
            .await?;
        Ok(result.as_str().unwrap_or("").to_string())
    }

    /// Remove download result
    pub async fn remove_download_result(
        &self,
        gid: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let result = self
            .call_method("aria2.removeDownloadResult", vec![json!(gid)])
            .await?;
        Ok(result.as_str().unwrap_or("OK").to_string())
    }

    /// Get global statistics
    pub async fn get_global_stat(&self) -> Result<Value, Box<dyn std::error::Error>> {
        self.call_method("aria2.getGlobalStat", vec![]).await
    }

    /// Purge download results
    pub async fn purge_download_result(&self) -> Result<String, Box<dyn std::error::Error>> {
        let result = self
            .call_method("aria2.purgeDownloadResult", vec![])
            .await?;
        Ok(result.as_str().unwrap_or("OK").to_string())
    }

    /// Get aria2 version
    pub async fn get_version(&self) -> Result<Value, Box<dyn std::error::Error>> {
        self.call_method("aria2.getVersion", vec![]).await
    }

    /// Shutdown aria2c
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.call_method("aria2.shutdown", vec![]).await;

        if let Some(mut child) = self.process.lock().await.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        Ok(())
    }
}

impl Drop for Aria2Manager {
    fn drop(&mut self) {
        // Note: We don't shutdown aria2c in drop to allow it to continue running
        // Users should explicitly call shutdown() if they want to stop aria2c
    }
}

// Helper module for base64 encoding
mod base64 {
    pub fn encode(data: &[u8]) -> String {
        use std::io::Write;
        let mut buf = Vec::new();
        {
            let mut encoder = base64::write::EncoderWriter::new(
                &mut buf,
                &base64::engine::general_purpose::STANDARD,
            );
            encoder.write_all(data).unwrap();
        }
        String::from_utf8(buf).unwrap()
    }
}
