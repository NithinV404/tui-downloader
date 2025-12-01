use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, widgets::ListState};

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
use ui::{PopupType, filter_by_tab, render_app, render_popup, render_size_warning};

// Minimum terminal size requirements
const MIN_WIDTH: u16 = 100;
const MIN_HEIGHT: u16 = 24;

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

    //Setup for terminal backend
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut list_state = ListState::default();
    list_state.select(Some(0));
    let mut current_tab: usize = 0;

    let mut input_handler = InputHandler::new();
    let status_message = Arc::new(RwLock::new(String::new()));
    let mut show_quit_confirm = false;
    let mut waiting_for_quit_response = false;

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
        let input_mode = input_handler.mode;
        let status_msg = status_message.read().await.clone();

        terminal.draw(|f| {
            let size = f.size();

            // Check terminal size
            if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
                render_size_warning(f, size, MIN_WIDTH, MIN_HEIGHT, size.width, size.height);
            } else {
                render_app(
                    f,
                    &all_downloads,
                    current_tab,
                    &mut list_state,
                    &input_text,
                    input_mode,
                    &status_msg,
                );

                // Show quit confirmation popup if requested
                if show_quit_confirm {
                    let active_count = all_downloads
                        .iter()
                        .filter(|d| d.status == "ACTIVE" || d.status == "WAITING")
                        .count();

                    let message = if active_count > 0 {
                        format!(
                            "You have {} active/queued download(s).\n\n\
                            Quitting will cancel all downloads.\n\n\
                            Are you sure you want to quit?",
                            active_count
                        )
                    } else {
                        "Are you sure you want to quit?".to_string()
                    };

                    render_popup(
                        f,
                        size,
                        "Confirm Quit",
                        &message,
                        PopupType::Confirmation,
                        true,
                    );
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

                    // Handle quit confirmation popup
                    if waiting_for_quit_response {
                        match key.code {
                            crossterm::event::KeyCode::Char('y')
                            | crossterm::event::KeyCode::Char('Y') => {
                                break;
                            }
                            crossterm::event::KeyCode::Char('n')
                            | crossterm::event::KeyCode::Char('N')
                            | crossterm::event::KeyCode::Esc => {
                                show_quit_confirm = false;
                                waiting_for_quit_response = false;
                            }
                            _ => {}
                        }
                        continue;
                    }

                    let action = input_handler.handle_key(&key);

                    match action {
                        KeyAction::EnterEditMode => {
                            input_handler.enter_edit_mode();
                        }
                        KeyAction::Quit => {
                            // Show confirmation popup instead of quitting immediately
                            show_quit_confirm = true;
                            waiting_for_quit_response = true;
                        }
                        KeyAction::SelectTab(tab) => {
                            current_tab = tab;
                            list_state.select(Some(0));
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
                        KeyAction::SubmitInput => {
                            if !input_handler.get_input().is_empty() {
                                let url = input_handler.take_input();

                                // Add download synchronously
                                let dm = download_manager.clone();
                                let url_clone = url.clone();
                                tokio::task::block_in_place(|| {
                                    tokio::runtime::Handle::current().block_on(async {
                                        let _ = dm.add_download(&url_clone).await;
                                    })
                                });

                                input_handler.exit_edit_mode();
                            } else {
                                input_handler.exit_edit_mode();
                            }
                        }
                        KeyAction::CancelInput => input_handler.exit_edit_mode(),
                        KeyAction::DeleteChar => input_handler.delete_last_char(),
                        KeyAction::PauseResume => {
                            if let Some(selected_idx) = list_state.selected() {
                                let filtered_downloads = filter_by_tab(&all_downloads, current_tab);
                                if selected_idx < filtered_downloads.len() {
                                    let download = filtered_downloads[selected_idx];
                                    if let Some(gid) = &download.gid {
                                        let dm = download_manager.clone();
                                        let gid_clone = gid.clone();
                                        tokio::task::block_in_place(|| {
                                            tokio::runtime::Handle::current().block_on(async {
                                                if download.status == "PAUSED" {
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
                                        let dm = download_manager.clone();
                                        let gid_clone = gid.clone();
                                        let status_msg = status_message.clone();

                                        tokio::task::block_in_place(|| {
                                            tokio::runtime::Handle::current().block_on(async {
                                                match dm.delete_file(&gid_clone).await {
                                                    Ok(msg) => {
                                                        *status_msg.write().await = msg;
                                                    }
                                                    Err(e) => {
                                                        *status_msg.write().await =
                                                            format!("Failed to delete file: {}", e);
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
                        KeyAction::PurgeCompleted => {
                            if current_tab == 2 {
                                // Only allow purging when on Completed tab
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
                            } else {
                                *status_message.write().await =
                                    "Switch to Completed tab to purge".to_string();
                            }
                        }
                        KeyAction::ClearAll => {
                            input_handler.buffer.clear();
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
            status_msg_clone.write().await.clear();
        });
    }

    // Cleanup
    download_manager.shutdown().await?;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
