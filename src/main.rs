use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, LineGauge, ListState, Paragraph, Tabs},
};
use std::io;

#[derive(Clone)]
struct Download {
    name: String,
    progress: f64,
    speed: String,
    status: String,
}

const APP_VERSION: &str = "1.0";

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
    let mut current_tab: usize = 0;
    let mut scroll_offset: usize = 0;

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

        // Update list_state if filtered list is smaller
        if list_state.selected().unwrap_or(0) >= filtered_downloads.len() {
            list_state.select(Some(filtered_downloads.len().saturating_sub(1)));
        }

        // Draw the UI
        terminal.draw(|f| {
            let size = f.size(); // Get the terminal size

            // Vertical Layout: Tabs, Main Area, Instructions
            let vertical_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Tabs
                    Constraint::Min(1),    // Main Area
                    Constraint::Length(1), // Instructions
                ])
                .split(size);

            //Tabs for filtering
            let tabs = Tabs::new(vec!["[1] Active", "[2] Queue", "[3] Completed"])
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("TUI Downloader {}", APP_VERSION)),
                )
                .select(current_tab)
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow));

            // Horizontal Layout for downloads and main area
            let horizontal_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(vertical_layout[1]);

            let selected_index = list_state.selected().unwrap_or(0);
            let list_area = horizontal_layout[0];

            // Calculate how many items can fit in the visible area
            // Each item takes 2 rows (name + gauge)
            let visible_rows = (list_area.height as usize).saturating_sub(2); // -2 for borders
            let items_per_screen = visible_rows / 2;

            // Auto-scroll to keep selected item visible
            if selected_index < scroll_offset {
                scroll_offset = selected_index;
            } else if selected_index >= scroll_offset + items_per_screen {
                scroll_offset = selected_index - items_per_screen + 1;
            }

            // Render the list with scrolling
            let mut current_y = list_area.top() + 1; // +1 for top border

            for (idx, item) in filtered_downloads.iter().enumerate() {
                // Skip items before scroll offset
                if idx < scroll_offset {
                    continue;
                }
                // Stop if we've filled the visible area
                if current_y + 2 > list_area.bottom() {
                    break;
                }

                let is_selected = idx == selected_index;

                // Create layout for this item (2 rows)
                let item_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Length(1)])
                    .split(ratatui::layout::Rect {
                        x: list_area.left() + 1,
                        y: current_y,
                        width: list_area.width.saturating_sub(2),
                        height: 2,
                    });

                // Name line
                let display_name = if item.name.len() > 25 {
                    format!("{}..", &item.name[0..24])
                } else {
                    item.name.clone()
                };

                let name_style = if is_selected {
                    Style::default().fg(Color::Yellow)
                } else if item.progress == 1.0{
                    Style::default().fg(Color::Green)
                }
                else{
                    Style::default().fg(Color::LightYellow)
                };

                let name_paragraph = Paragraph::new(display_name).style(name_style);
                f.render_widget(name_paragraph, item_layout[0]);

                // Gauge line
                let gauge_label = Line::from(vec![
                    Span::raw(format!("{:.0}% • ", item.progress * 100.0)),
                    Span::raw(&item.speed),
                ]);

                let gauge = LineGauge::default()
                    .ratio(item.progress)
                    .label(gauge_label)
                    .gauge_style(Style::default().fg(if is_selected {
                        Color::Yellow
                    } else if item.progress == 1.0{
                        Color::Green
                    }
                    else{
                        Color::LightYellow
                    }));

                f.render_widget(gauge, item_layout[1]);

                current_y += 2;
            }

            // Render list border
            let list_border = Block::default()
                .borders(Borders::ALL)
                .title(format!("Downloads [{}/{}]", selected_index + 1, filtered_downloads.len()));
            f.render_widget(list_border, list_area);

            // Details Pane
            let selected_download = if selected_index < filtered_downloads.len() {
                filtered_downloads[selected_index].clone()
            } else {
                downloads[0].clone()
            };

            let status_text = format!("STATUS: {}\n\nFile: {}\nSize: 4.7 GB\nServer: mirrors.edge.org\nPath: ~/Downloads/ISOs", selected_download.status, selected_download.name);
            let logs_text = "> Handshake successful\n> Allocating disk space...\n> Connected to peer 1\n> Connected to peer 2\n> Chunk 1452 verified\n> Rate limit: None";

            let details = Paragraph::new(status_text)
                .block(Block::default().borders(Borders::ALL).title("Details"));
            let logs = Paragraph::new(logs_text)
                .block(Block::default().borders(Borders::ALL).title("LOGS"));

            // Vertical split for details/logs
            let right_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(horizontal_layout[1]);

            // Instructions with better contrast and styling
            let instructions = Paragraph::new("<Space> Pause/Resume   <D> Delete   <Enter> Open Details   <Q> Quit")
                .style(Style::default().bg(Color::Cyan).fg(Color::Black))
                .alignment(Alignment::Center);

            // Render widgets
            f.render_widget(tabs, vertical_layout[0]);
            f.render_widget(details, right_layout[0]);
            f.render_widget(logs, right_layout[1]);
            f.render_widget(instructions, vertical_layout[2]);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    crossterm::event::KeyCode::Char('q') => break,
                    crossterm::event::KeyCode::Char('1') => current_tab = 0,
                    crossterm::event::KeyCode::Char('2') => current_tab = 1,
                    crossterm::event::KeyCode::Char('3') => current_tab = 2,
                    crossterm::event::KeyCode::Up => {
                        let i = list_state.selected().unwrap_or(0);
                        if i > 0 {
                            list_state.select(Some(i - 1));
                        }
                    }
                    crossterm::event::KeyCode::Down => {
                        let i = list_state.selected().unwrap_or(0);
                        if i < filtered_downloads.len().saturating_sub(1) {
                            list_state.select(Some(i + 1));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
