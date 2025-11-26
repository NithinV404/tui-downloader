use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::ListState,
};
use std::io;

mod input;
mod models;
mod ui;

use input::{InputHandler, KeyAction};
use models::{Download, InputMode};
use ui::render_input_field;

use crate::ui::{render_details_pane, render_downloads_list, render_input_guide, render_tabs};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let downloads = vec![
        Download {
            name: "ubuntu-22.04.iso".to_string(),
            progress: 0.6,
            speed: "5.2MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "holiday_photos_2023.zip".to_string(),
            progress: 1.0,
            speed: "Done".to_string(),
            status: "COMPLETED".to_string(),
        },
        Download {
            name: "large_dataset_v2.csv".to_string(),
            progress: 0.15,
            speed: "1.1MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "music_backup.tar.gz".to_string(),
            progress: 0.0,
            speed: "Pending".to_string(),
            status: "QUEUED".to_string(),
        },
        Download {
            name: "nodejs-18.15.0.tar.gz".to_string(),
            progress: 0.45,
            speed: "3.8MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "docker-image-backup.tar".to_string(),
            progress: 0.0,
            speed: "Pending".to_string(),
            status: "QUEUED".to_string(),
        },
        Download {
            name: "python-3.11-docs.pdf".to_string(),
            progress: 0.82,
            speed: "2.1MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "rust-1.70.0-installer.exe".to_string(),
            progress: 1.0,
            speed: "Done".to_string(),
            status: "COMPLETED".to_string(),
        },
        Download {
            name: "game-assets-latest.zip".to_string(),
            progress: 0.33,
            speed: "4.5MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "database-backup-2024.sql".to_string(),
            progress: 0.0,
            speed: "Pending".to_string(),
            status: "QUEUED".to_string(),
        },
        Download {
            name: "video-tutorial-4k.mkv".to_string(),
            progress: 0.25,
            speed: "8.7MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "scientific-papers-bundle.tar.gz".to_string(),
            progress: 0.55,
            speed: "2.3MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "minecraft-world-save.zip".to_string(),
            progress: 0.67,
            speed: "6.2MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "photoshop-plugin-pack.rar".to_string(),
            progress: 0.0,
            speed: "Pending".to_string(),
            status: "QUEUED".to_string(),
        },
        Download {
            name: "blender-project-files.blend".to_string(),
            progress: 0.91,
            speed: "5.5MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "kubernetes-deployment.yaml".to_string(),
            progress: 1.0,
            speed: "Done".to_string(),
            status: "COMPLETED".to_string(),
        },
        Download {
            name: "audio-track-master.wav".to_string(),
            progress: 0.38,
            speed: "3.2MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "font-collection-complete.zip".to_string(),
            progress: 0.0,
            speed: "Pending".to_string(),
            status: "QUEUED".to_string(),
        },
        Download {
            name: "virtual-machine-image.iso".to_string(),
            progress: 0.72,
            speed: "7.1MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "machine-learning-dataset.csv".to_string(),
            progress: 0.19,
            speed: "2.8MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "graphic-design-mockups.psd".to_string(),
            progress: 1.0,
            speed: "Done".to_string(),
            status: "COMPLETED".to_string(),
        },
        Download {
            name: "source-code-repository.git".to_string(),
            progress: 0.44,
            speed: "4.3MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "system-backup-full.backup".to_string(),
            progress: 0.0,
            speed: "Pending".to_string(),
            status: "QUEUED".to_string(),
        },
        Download {
            name: "streaming-video-hd.mp4".to_string(),
            progress: 0.58,
            speed: "9.4MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "cryptography-library.jar".to_string(),
            progress: 1.0,
            speed: "Done".to_string(),
            status: "COMPLETED".to_string(),
        },
        Download {
            name: "weather-station-data.json".to_string(),
            progress: 0.26,
            speed: "1.5MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "mobile-app-build.apk".to_string(),
            progress: 0.0,
            speed: "Pending".to_string(),
            status: "QUEUED".to_string(),
        },
        Download {
            name: "embedded-firmware.bin".to_string(),
            progress: 0.85,
            speed: "1.9MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "web-scraper-output.xlsx".to_string(),
            progress: 0.41,
            speed: "3.6MB/s".to_string(),
            status: "DOWNLOADING".to_string(),
        },
        Download {
            name: "security-audit-report.pdf".to_string(),
            progress: 1.0,
            speed: "Done".to_string(),
            status: "COMPLETED".to_string(),
        },
    ];

    /*┌─ TUI Downloader v1.0 ───────────────────────────────────────────────┐
    │  Tabs: [1] Active (3)   [2] Queue      [3] Completed                │
    ├───────────────────────────────────────┬─────────────────────────────┤
    │                                       │                             │
    │  1. ubuntu-22.04.iso                  │  STATUS: DOWNLOADING        │
    │     [██████████░░░░░░] 60% • 5.2MB/s  │                             │
    │                                       │  File: ubuntu-22.04.iso     │
    │                                       │  Size: 4.7 GB               │
    │  2. holiday_photos_2023.zip           │  Server: mirrors.edge.org   │
    │     [████████████████] 100% • Done    │  Path: ~/Downloads/ISOs     │
    │                                       │                             │
    │ >>  3. large_dataset_v2.csv           │  LOGS ────────────────────  │
    │     [███░░░░░░░░░░░░░] 15% • 1.1MB/s  │  > Handshake successful     │
    │                                       │  > Allocating disk space... │
    │                                       │  > Connected to peer 1      │
    │  4. music_backup.tar.gz               │  > Connected to peer 2      │
    │     [░░░░░░░░░░░░░░░░] 0%  • Pending  │  > Chunk 1452 verified      │
    │                                       │  > Rate limit: None         │
    │                                       │                             │
    │                                       │                             │
    │                                       │                             │
    └───────────────────────────────────────┴─────────────────────────────┘
    <Space> Pause/Resume   <D> Delete   <Enter> Open Details   <Q> Quit     */

    //Setup for terminal backend
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut list_state = ListState::default();
    list_state.select(Some(0));
    let selected_index = list_state.selected().unwrap_or(0);
    let mut current_tab: usize = 0;

    let mut url_input = String::new();
    let mut input_handler = InputHandler::new();
    let mut input_mode = InputMode::Normal;

    // Main loop
    loop {
        let filtered_downloads: Vec<&Download> = downloads
            .iter()
            .filter(|d| match current_tab {
                0 => d.progress > 0.0 && d.progress < 1.0,
                1 => d.progress == 0.0,
                2 => d.progress == 1.0,
                _ => false,
            })
            .collect();

        // Draw the UI
        terminal.draw(|f| {
            let size = f.size(); // Get the terminal size

            // Vertical Layout: Tabs, Main Area, Input Guide
            let vertical_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Input for links
                    Constraint::Length(3), // Tabs
                    Constraint::Min(1),    // Main Area
                    Constraint::Length(1), // Instructions
                ])
                .split(size);

            // Horizontal Layout: Downloads_List, Download_Info
            let horizontal_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(vertical_layout[2]);

            // Input Field Pane UI
            render_input_field(f, vertical_layout[0], &url_input, input_mode);

            // Tabs Field Pane UI
            let tab_titles = vec!["[1] Active", "[2] Queue", "[3] Completed"];
            render_tabs(f, vertical_layout[1], current_tab, tab_titles);

            // Render Downloads Pane UI
            render_downloads_list(
                f,
                horizontal_layout[0],
                &filtered_downloads,
                &mut list_state,
            );

            // Render Details Pane UI
            let selected_download = if selected_index < filtered_downloads.len() {
                filtered_downloads[selected_index].clone()
            } else {
                downloads[0].clone()
            };
            render_details_pane(f, horizontal_layout[1], selected_download);

            // Render Instructions Pane UI
            render_input_guide(f, vertical_layout[3], input_mode);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key) => {
                    let action = input_handler.handle_key(&key);

                    match action {
                        KeyAction::EnterEditMode => input_handler.enter_edit_mode(),
                        KeyAction::Quit => break,
                        KeyAction::SelectTab(tab) => current_tab = tab,
                        KeyAction::MoveUp => {
                            let i = list_state.selected().unwrap_or(0);
                            if i > 0 {
                                list_state.select(Some(i - 1));
                            }
                        }
                        KeyAction::MoveDown => {
                            let i = list_state.selected().unwrap_or(0);
                            if i < filtered_downloads.len().saturating_sub(1) {
                                list_state.select(Some(i + 1));
                            }
                        }
                        KeyAction::SubmitInput => {
                            if !input_handler.get_input().is_empty() {
                                let url = input_handler.take_input();
                                // TODO: Process the URL
                                input_handler.exit_edit_mode();
                            } else {
                                input_handler.exit_edit_mode();
                            }
                        }
                        KeyAction::CancelInput => input_handler.exit_edit_mode(),
                        KeyAction::DeleteChar => input_handler.delete_last_char(),
                        KeyAction::ClearAll => {}
                        KeyAction::None => {}
                    }
                }
                crossterm::event::Event::Paste(data) => {
                    input_handler.handle_paste(&data);
                }
                _ => {}
            }
        }
        url_input = input_handler.get_input().to_string();
        input_mode = input_handler.mode;
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
