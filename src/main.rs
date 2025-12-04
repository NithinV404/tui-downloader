use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};

use std::io;
use std::sync::Arc;
use tokio::sync::RwLock;

mod aria2;
mod download_manager;
mod input;
mod models;
mod ui;

use download_manager::DownloadManager;
use input::{InputHandler, KeyAction};
use models::{ConfirmAction, InputMode};
use ui::{
    filter_by_tab, render_app_full, render_popup, render_size_warning, AppState, PopupType,
    SortOrder, SpeedLimitState,
};

// Minimum terminal size requirements
const MIN_WIDTH: u16 = 100;
const MIN_HEIGHT: u16 = 30;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize download manager (this will auto-spawn aria2c)
    let download_manager = match DownloadManager::new().await {
        Ok(dm) => Arc::new(dm),
        Err(e) => {
            eprintln!("Failed to initialize download manager: {}", e);
            eprintln!("Make sure aria2c is installed on your system.");
            eprintln!("You can install it with:");
            eprintln!("  - Ubuntu/Debian: sudo apt install aria2");
            eprintln!("  - Fedora: sudo dnf install aria2");
            eprintln!("  - Arch: sudo pacman -S aria2");
            eprintln!("  - macOS: brew install aria2");
            return Err(e);
        }
    };

    // Setup for terminal backend
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Application state
    let mut list_state = ListState::default();
    list_state.select(Some(0));
    let mut current_tab: usize = 0;

    let mut input_handler = InputHandler::new();
    let status_message = Arc::new(RwLock::new(String::new()));

    // New feature states
    let mut sort_order = SortOrder::Name;
    let mut sort_ascending = true;
    let mut help_scroll: usize = 0;
    let mut speed_limit_state = SpeedLimitState::default();
    let mut download_limit: u64 = 0;
    let mut upload_limit: u64 = 0;
    let mut selected_indices: Vec<usize> = Vec::new();
    let mut pending_confirm: Option<ConfirmAction> = None;

    // Spawn background task to update downloads from aria2c
    let dm_clone = download_manager.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = dm_clone.update_downloads().await {
                eprintln!("Error updating downloads: {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    // Main loop
    loop {
        // Get downloads from manager
        let all_downloads = download_manager.get_all_downloads().await;

        // Draw the UI using the modular render function
        let input_text = input_handler.get_input().to_string();
        let search_query = input_handler.get_search_query().to_string();
        let input_mode = input_handler.mode;
        let status_msg = status_message.read().await.clone();

        terminal.draw(|f| {
            let size = f.size();

            // Check terminal size
            if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
                render_size_warning(f, size, MIN_WIDTH, MIN_HEIGHT, size.width, size.height);
            } else {
                // Build app state with all features
                let state = AppState {
                    downloads: &all_downloads,
                    current_tab,
                    input_text: &input_text,
                    input_mode,
                    status_message: &status_msg,
                    search_query: &search_query,
                    sort_order,
                    sort_ascending,
                    help_scroll,
                    speed_limit_state: if input_mode == InputMode::SpeedLimit {
                        Some(&speed_limit_state)
                    } else {
                        None
                    },
                    download_limit,
                    upload_limit,
                    selected_indices: &selected_indices,
                };

                render_app_full(f, state, &mut list_state);

                // Show confirmation popup if pending
                if let Some(ref action) = pending_confirm {
                    let (title, message) = match action {
                        ConfirmAction::Quit => {
                            let active_count = all_downloads
                                .iter()
                                .filter(|d| d.status == "ACTIVE" || d.status == "WAITING")
                                .count();

                            let msg = if active_count > 0 {
                                format!(
                                    "You have {} active/queued download(s).\n\n\
                                    Quitting will cancel all downloads.\n\n\
                                    Are you sure you want to quit?",
                                    active_count
                                )
                            } else {
                                "Are you sure you want to quit?".to_string()
                            };
                            ("Confirm Quit", msg)
                        }
                        ConfirmAction::DeleteFile(gid) => {
                            let name = all_downloads
                                .iter()
                                .find(|d| d.gid.as_ref() == Some(gid))
                                .map(|d| d.name.clone())
                                .unwrap_or_else(|| "Unknown".to_string());
                            (
                                "Delete File",
                                format!(
                                    "Are you sure you want to delete this file from disk?\n\n\
                                    File: {}\n\n\
                                    This cannot be undone!",
                                    name
                                ),
                            )
                        }
                        ConfirmAction::PurgeCompleted => (
                            "Purge Completed",
                            "Are you sure you want to remove all completed downloads from the list?"
                                .to_string(),
                        ),
                        ConfirmAction::RetryDownload(_) => (
                            "Retry Download",
                            "Retry this failed download?".to_string(),
                        ),
                    };

                    render_popup(f, size, title, &message, PopupType::Confirmation, true);
                }
            }
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key) => {
                    // Check terminal size and allow force quit
                    let size = terminal.size()?;
                    if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
                        // Only allow quit when terminal is too small
                        if let crossterm::event::KeyCode::Char('q')
                        | crossterm::event::KeyCode::Char('Q') = key.code
                        {
                            break;
                        }
                        continue;
                    }

                    // Handle confirmation popup responses
                    if pending_confirm.is_some() {
                        match key.code {
                            crossterm::event::KeyCode::Char('y')
                            | crossterm::event::KeyCode::Char('Y') => {
                                if let Some(action) = pending_confirm.take() {
                                    match action {
                                        ConfirmAction::Quit => {
                                            break;
                                        }
                                        ConfirmAction::DeleteFile(gid) => {
                                            let dm = download_manager.clone();
                                            let status_msg = status_message.clone();

                                            tokio::task::block_in_place(|| {
                                                tokio::runtime::Handle::current().block_on(async {
                                                    match dm.delete_file(&gid).await {
                                                        Ok(msg) => {
                                                            *status_msg.write().await = msg;
                                                        }
                                                        Err(e) => {
                                                            *status_msg.write().await = format!(
                                                                "Failed to delete file: {}",
                                                                e
                                                            );
                                                        }
                                                    }
                                                })
                                            });
                                        }
                                        ConfirmAction::PurgeCompleted => {
                                            let dm = download_manager.clone();
                                            let status_msg = status_message.clone();

                                            tokio::task::block_in_place(|| {
                                                tokio::runtime::Handle::current().block_on(async {
                                                    match dm.purge_completed().await {
                                                        Ok(count) => {
                                                            *status_msg.write().await = format!(
                                                                "Purged {} completed download(s)",
                                                                count
                                                            );
                                                        }
                                                        Err(e) => {
                                                            *status_msg.write().await =
                                                                format!("Purge failed: {}", e);
                                                        }
                                                    }
                                                })
                                            });
                                            list_state.select(None);
                                        }
                                        ConfirmAction::RetryDownload(gid) => {
                                            let dm = download_manager.clone();
                                            let status_msg = status_message.clone();

                                            tokio::task::block_in_place(|| {
                                                tokio::runtime::Handle::current().block_on(async {
                                                    match dm.retry_download(&gid).await {
                                                        Ok(_) => {
                                                            *status_msg.write().await =
                                                                "Download restarted".to_string();
                                                        }
                                                        Err(e) => {
                                                            *status_msg.write().await =
                                                                format!("Failed to retry: {}", e);
                                                        }
                                                    }
                                                })
                                            });
                                        }
                                    }
                                }
                                input_handler.exit_to_normal();
                            }
                            crossterm::event::KeyCode::Char('n')
                            | crossterm::event::KeyCode::Char('N')
                            | crossterm::event::KeyCode::Esc => {
                                pending_confirm = None;
                                input_handler.exit_to_normal();
                            }
                            _ => {}
                        }
                        continue;
                    }

                    let action = input_handler.handle_key(&key);

                    match action {
                        // ============ Normal Mode Actions ============
                        KeyAction::EnterEditMode => {
                            input_handler.enter_edit_mode();
                        }
                        KeyAction::Quit => {
                            pending_confirm = Some(ConfirmAction::Quit);
                            input_handler.enter_confirmation_mode();
                        }
                        KeyAction::SelectTab(tab) => {
                            current_tab = tab;
                            list_state.select(Some(0));
                            selected_indices.clear();
                        }
                        KeyAction::MoveUp => {
                            let i = list_state.selected().unwrap_or(0);
                            if i > 0 {
                                list_state.select(Some(i - 1));
                            }
                        }
                        KeyAction::MoveDown => {
                            let i = list_state.selected().unwrap_or(0);
                            let filtered_count = filter_by_tab(&all_downloads, current_tab).len();
                            if i < filtered_count.saturating_sub(1) {
                                list_state.select(Some(i + 1));
                            }
                        }
                        KeyAction::MoveToTop => {
                            list_state.select(Some(0));
                        }
                        KeyAction::MoveToBottom => {
                            let filtered_count = filter_by_tab(&all_downloads, current_tab).len();
                            if filtered_count > 0 {
                                list_state.select(Some(filtered_count - 1));
                            }
                        }
                        KeyAction::PageUp => {
                            let i = list_state.selected().unwrap_or(0);
                            let new_i = i.saturating_sub(10);
                            list_state.select(Some(new_i));
                        }
                        KeyAction::PageDown => {
                            let i = list_state.selected().unwrap_or(0);
                            let filtered_count = filter_by_tab(&all_downloads, current_tab).len();
                            let new_i = (i + 10).min(filtered_count.saturating_sub(1));
                            list_state.select(Some(new_i));
                        }

                        // ============ Search Actions ============
                        KeyAction::EnterSearchMode => {
                            input_handler.enter_search_mode();
                        }
                        KeyAction::ClearSearch => {
                            input_handler.clear_search();
                        }
                        KeyAction::SearchSubmit | KeyAction::SearchCancel => {
                            input_handler.exit_to_normal();
                        }
                        KeyAction::SearchDeleteChar => {
                            // Already handled in input handler
                        }

                        // ============ Help Actions ============
                        KeyAction::ShowHelp => {
                            input_handler.enter_help_mode();
                            help_scroll = 0;
                        }
                        KeyAction::HelpClose => {
                            input_handler.exit_to_normal();
                        }
                        KeyAction::HelpScrollUp => {
                            help_scroll = help_scroll.saturating_sub(1);
                        }
                        KeyAction::HelpScrollDown => {
                            help_scroll += 1;
                        }

                        // ============ Speed Limit Actions ============
                        KeyAction::ShowSpeedLimit => {
                            // Get current limits from aria2
                            let dm = download_manager.clone();
                            tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(async {
                                    if let Ok((dl, ul)) = dm.get_speed_limits().await {
                                        speed_limit_state = SpeedLimitState::new(dl, ul);
                                        download_limit = dl;
                                        upload_limit = ul;
                                    }
                                })
                            });
                            input_handler.enter_speed_limit_mode();
                        }
                        KeyAction::SpeedLimitConfirm => {
                            // Apply speed limits
                            let dm = download_manager.clone();
                            let dl = speed_limit_state.download_limit;
                            let ul = speed_limit_state.upload_limit;
                            let status_msg = status_message.clone();

                            tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(async {
                                    let _ = dm.set_download_speed_limit(dl).await;
                                    let _ = dm.set_upload_speed_limit(ul).await;
                                    download_limit = dl;
                                    upload_limit = ul;

                                    let dl_str = if dl == 0 {
                                        "Unlimited".to_string()
                                    } else {
                                        ui::format_speed(dl)
                                    };
                                    let ul_str = if ul == 0 {
                                        "Unlimited".to_string()
                                    } else {
                                        ui::format_speed(ul)
                                    };
                                    *status_msg.write().await =
                                        format!("Speed limits set: D:{} U:{}", dl_str, ul_str);
                                })
                            });
                            input_handler.exit_to_normal();
                        }
                        KeyAction::SpeedLimitCancel => {
                            input_handler.exit_to_normal();
                        }
                        KeyAction::SpeedLimitToggleField => {
                            speed_limit_state.toggle_field();
                        }
                        KeyAction::SpeedLimitIncrease => {
                            speed_limit_state.increase_limit();
                        }
                        KeyAction::SpeedLimitDecrease => {
                            speed_limit_state.decrease_limit();
                        }

                        // ============ Sorting Actions ============
                        KeyAction::CycleSort => {
                            sort_order = sort_order.next();
                            *status_message.write().await =
                                format!("Sort by: {}", sort_order.as_str());
                        }
                        KeyAction::ToggleSortDirection => {
                            sort_ascending = !sort_ascending;
                            let dir = if sort_ascending {
                                "Ascending"
                            } else {
                                "Descending"
                            };
                            *status_message.write().await = format!("Sort direction: {}", dir);
                        }

                        // ============ Download Management ============
                        KeyAction::SubmitInput => {
                            if !input_handler.get_input().is_empty() {
                                let url = input_handler.take_input();
                                let dm = download_manager.clone();
                                let status_msg = status_message.clone();

                                tokio::task::block_in_place(|| {
                                    tokio::runtime::Handle::current().block_on(async {
                                        match dm.add_download(&url).await {
                                            Ok(_) => {
                                                *status_msg.write().await =
                                                    "Download added".to_string();
                                            }
                                            Err(e) => {
                                                *status_msg.write().await =
                                                    format!("Failed to add download: {}", e);
                                            }
                                        }
                                    })
                                });

                                input_handler.exit_edit_mode();
                            } else {
                                input_handler.exit_edit_mode();
                            }
                        }
                        KeyAction::CancelInput => {
                            input_handler.exit_edit_mode();
                        }
                        KeyAction::DeleteChar
                        | KeyAction::DeleteWord
                        | KeyAction::MoveCursorLeft
                        | KeyAction::MoveCursorRight
                        | KeyAction::MoveCursorStart
                        | KeyAction::MoveCursorEnd => {
                            // Already handled in input handler
                        }
                        KeyAction::ClearAll => {
                            input_handler.buffer.clear();
                        }

                        KeyAction::PauseResume => {
                            if let Some(selected_idx) = list_state.selected() {
                                let filtered_downloads = filter_by_tab(&all_downloads, current_tab);
                                if selected_idx < filtered_downloads.len() {
                                    let download = filtered_downloads[selected_idx];
                                    if let Some(gid) = &download.gid {
                                        let dm = download_manager.clone();
                                        let gid_clone = gid.clone();
                                        let is_paused = download.status == "PAUSED";

                                        tokio::task::block_in_place(|| {
                                            tokio::runtime::Handle::current().block_on(async {
                                                if is_paused {
                                                    let _ = dm.resume_download(&gid_clone).await;
                                                } else {
                                                    let _ = dm.pause_download(&gid_clone).await;
                                                }
                                            })
                                        });
                                    }
                                }
                            }
                        }
                        KeyAction::PauseAll => {
                            let dm = download_manager.clone();
                            let status_msg = status_message.clone();

                            tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(async {
                                    match dm.pause_all().await {
                                        Ok(_) => {
                                            *status_msg.write().await =
                                                "All downloads paused".to_string();
                                        }
                                        Err(e) => {
                                            *status_msg.write().await =
                                                format!("Failed to pause all: {}", e);
                                        }
                                    }
                                })
                            });
                        }
                        KeyAction::ResumeAll => {
                            let dm = download_manager.clone();
                            let status_msg = status_message.clone();

                            tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(async {
                                    match dm.resume_all().await {
                                        Ok(_) => {
                                            *status_msg.write().await =
                                                "All downloads resumed".to_string();
                                        }
                                        Err(e) => {
                                            *status_msg.write().await =
                                                format!("Failed to resume all: {}", e);
                                        }
                                    }
                                })
                            });
                        }
                        KeyAction::Delete => {
                            if let Some(selected_idx) = list_state.selected() {
                                let filtered_downloads = filter_by_tab(&all_downloads, current_tab);
                                if selected_idx < filtered_downloads.len() {
                                    let download = filtered_downloads[selected_idx];
                                    if let Some(gid) = &download.gid {
                                        let dm = download_manager.clone();
                                        let gid_clone = gid.clone();
                                        let download_name = download.name.clone();
                                        let status_msg = status_message.clone();

                                        tokio::task::block_in_place(|| {
                                            tokio::runtime::Handle::current().block_on(async {
                                                match dm.remove_download(&gid_clone).await {
                                                    Ok(_) => {
                                                        *status_msg.write().await =
                                                            format!("Deleted: {}", download_name);
                                                    }
                                                    Err(e) => {
                                                        *status_msg.write().await =
                                                            format!("Delete failed: {}", e);
                                                    }
                                                }
                                            })
                                        });

                                        // Adjust selection after deletion
                                        if selected_idx > 0 {
                                            list_state.select(Some(selected_idx - 1));
                                        } else if filtered_downloads.len() > 1 {
                                            list_state.select(Some(0));
                                        } else {
                                            list_state.select(None);
                                        }
                                    }
                                }
                            }
                        }
                        KeyAction::DeleteFile => {
                            if let Some(selected_idx) = list_state.selected() {
                                let filtered_downloads = filter_by_tab(&all_downloads, current_tab);
                                if selected_idx < filtered_downloads.len() {
                                    let download = filtered_downloads[selected_idx];
                                    if let Some(gid) = &download.gid {
                                        pending_confirm =
                                            Some(ConfirmAction::DeleteFile(gid.clone()));
                                        input_handler.enter_confirmation_mode();
                                    }
                                }
                            }
                        }
                        KeyAction::RetryDownload => {
                            if let Some(selected_idx) = list_state.selected() {
                                let filtered_downloads = filter_by_tab(&all_downloads, current_tab);
                                if selected_idx < filtered_downloads.len() {
                                    let download = filtered_downloads[selected_idx];
                                    if download.status == "ERROR" {
                                        if let Some(gid) = &download.gid {
                                            pending_confirm =
                                                Some(ConfirmAction::RetryDownload(gid.clone()));
                                            input_handler.enter_confirmation_mode();
                                        }
                                    } else {
                                        *status_message.write().await =
                                            "Can only retry failed downloads".to_string();
                                    }
                                }
                            }
                        }
                        KeyAction::PurgeCompleted => {
                            if current_tab == 2 {
                                pending_confirm = Some(ConfirmAction::PurgeCompleted);
                                input_handler.enter_confirmation_mode();
                            } else {
                                *status_message.write().await =
                                    "Switch to Completed tab to purge".to_string();
                            }
                        }

                        // ============ Queue Management ============
                        KeyAction::MoveQueueUp => {
                            if let Some(selected_idx) = list_state.selected() {
                                let filtered_downloads = filter_by_tab(&all_downloads, current_tab);
                                if selected_idx < filtered_downloads.len() {
                                    let download = filtered_downloads[selected_idx];
                                    if let Some(gid) = &download.gid {
                                        let dm = download_manager.clone();
                                        let gid_clone = gid.clone();

                                        tokio::task::block_in_place(|| {
                                            tokio::runtime::Handle::current().block_on(async {
                                                let _ = dm.move_up(&gid_clone).await;
                                            })
                                        });
                                    }
                                }
                            }
                        }
                        KeyAction::MoveQueueDown => {
                            if let Some(selected_idx) = list_state.selected() {
                                let filtered_downloads = filter_by_tab(&all_downloads, current_tab);
                                if selected_idx < filtered_downloads.len() {
                                    let download = filtered_downloads[selected_idx];
                                    if let Some(gid) = &download.gid {
                                        let dm = download_manager.clone();
                                        let gid_clone = gid.clone();

                                        tokio::task::block_in_place(|| {
                                            tokio::runtime::Handle::current().block_on(async {
                                                let _ = dm.move_down(&gid_clone).await;
                                            })
                                        });
                                    }
                                }
                            }
                        }

                        // ============ File Operations ============
                        KeyAction::OpenFile => {
                            if let Some(selected_idx) = list_state.selected() {
                                let filtered_downloads = filter_by_tab(&all_downloads, current_tab);
                                if selected_idx < filtered_downloads.len() {
                                    let download = filtered_downloads[selected_idx];
                                    if let Some(path) = &download.file_path {
                                        if let Err(e) = open::that(path) {
                                            *status_message.write().await =
                                                format!("Failed to open file: {}", e);
                                        }
                                    } else {
                                        *status_message.write().await =
                                            "File path not available".to_string();
                                    }
                                }
                            }
                        }
                        KeyAction::OpenFolder => {
                            if let Some(selected_idx) = list_state.selected() {
                                let filtered_downloads = filter_by_tab(&all_downloads, current_tab);
                                if selected_idx < filtered_downloads.len() {
                                    let download = filtered_downloads[selected_idx];
                                    if let Some(path) = &download.file_path {
                                        if let Some(parent) = std::path::Path::new(path).parent() {
                                            if let Err(e) = open::that(parent) {
                                                *status_message.write().await =
                                                    format!("Failed to open folder: {}", e);
                                            }
                                        }
                                    } else {
                                        // Open default download directory
                                        let download_dir = dirs::download_dir()
                                            .or_else(|| {
                                                dirs::home_dir().map(|p| p.join("Downloads"))
                                            })
                                            .unwrap_or_else(|| {
                                                std::path::PathBuf::from("./Downloads")
                                            });
                                        if let Err(e) = open::that(&download_dir) {
                                            *status_message.write().await =
                                                format!("Failed to open folder: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        KeyAction::CopyUrl => {
                            if let Some(selected_idx) = list_state.selected() {
                                let filtered_downloads = filter_by_tab(&all_downloads, current_tab);
                                if selected_idx < filtered_downloads.len() {
                                    let download = filtered_downloads[selected_idx];
                                    if let Some(url) = &download.url {
                                        // Try to copy to clipboard using cli-clipboard or arboard
                                        #[cfg(feature = "clipboard")]
                                        {
                                            if let Ok(mut ctx) = arboard::Clipboard::new() {
                                                if ctx.set_text(url.clone()).is_ok() {
                                                    *status_message.write().await =
                                                        "URL copied to clipboard".to_string();
                                                }
                                            }
                                        }
                                        #[cfg(not(feature = "clipboard"))]
                                        {
                                            *status_message.write().await = format!("URL: {}", url);
                                        }
                                    } else {
                                        *status_message.write().await =
                                            "URL not available".to_string();
                                    }
                                }
                            }
                        }
                        KeyAction::CopyPath => {
                            if let Some(selected_idx) = list_state.selected() {
                                let filtered_downloads = filter_by_tab(&all_downloads, current_tab);
                                if selected_idx < filtered_downloads.len() {
                                    let download = filtered_downloads[selected_idx];
                                    if let Some(path) = &download.file_path {
                                        #[cfg(feature = "clipboard")]
                                        {
                                            if let Ok(mut ctx) = arboard::Clipboard::new() {
                                                if ctx.set_text(path.clone()).is_ok() {
                                                    *status_message.write().await =
                                                        "Path copied to clipboard".to_string();
                                                }
                                            }
                                        }
                                        #[cfg(not(feature = "clipboard"))]
                                        {
                                            *status_message.write().await =
                                                format!("Path: {}", path);
                                        }
                                    } else {
                                        *status_message.write().await =
                                            "Path not available".to_string();
                                    }
                                }
                            }
                        }

                        // ============ Selection (Batch Operations) ============
                        KeyAction::ToggleSelect => {
                            if let Some(selected_idx) = list_state.selected() {
                                if selected_indices.contains(&selected_idx) {
                                    selected_indices.retain(|&i| i != selected_idx);
                                } else {
                                    selected_indices.push(selected_idx);
                                }
                            }
                        }
                        KeyAction::SelectAll => {
                            let filtered_count = filter_by_tab(&all_downloads, current_tab).len();
                            selected_indices = (0..filtered_count).collect();
                            *status_message.write().await =
                                format!("Selected {} items", filtered_count);
                        }
                        KeyAction::DeselectAll => {
                            selected_indices.clear();
                            *status_message.write().await = "Selection cleared".to_string();
                        }

                        // ============ Confirmation Mode ============
                        KeyAction::ConfirmYes | KeyAction::ConfirmNo => {
                            // Handled above in confirmation popup section
                        }

                        KeyAction::None => {}
                    }
                }
                crossterm::event::Event::Paste(data) => {
                    input_handler.handle_paste(&data);
                }
                _ => {}
            }
        }

        // Clear status message after 3 seconds
        let status_msg_clone = status_message.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            let mut msg = status_msg_clone.write().await;
            if !msg.is_empty() {
                msg.clear();
            }
        });
    }

    // Cleanup
    download_manager.shutdown().await?;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
