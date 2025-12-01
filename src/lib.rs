pub mod aria2;
pub mod download_manager;
pub mod input;
pub mod models;
pub mod ui;

pub use aria2::Aria2Manager;
pub use download_manager::DownloadManager;
pub use models::{Download, DownloadType, InputMode};
