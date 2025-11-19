use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io;

#[derive(Clone)]
struct Download {
    name: String,
    progress: f64, // 0.0 to 1.0
}

fn progress_bar(progress: f64, width: usize) -> String {
    let filled = (progress * width as f64).round() as usize;
    let empty = width - filled;
    let filled_chars = "█".repeat(filled);
    let empty_chars = "░".repeat(empty);
    format!("[{}{}] {:.0}%", filled_chars, empty_chars, progress * 100.0)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    let downloads = vec![
        Download {
            name: "This is a very big test item".to_string(),
            progress: 0.3,
        },
        Download {
            name: "Item 2".to_string(),
            progress: 0.7,
        },
        Download {
            name: "Item 3".to_string(),
            progress: 0.5,
        },
    ];
    /*--------------------------------------------------------------------------------------------------------
     The gui for application is handled below

    -Downloads-------------------------------------------------------------------------------------------------
     Item 1 [████████░░░░░░░░] 30%
     Item 2 [██████████████░░] 70%
     Item 3 [██████████░░░░░░] 50%

     -Info-----------------------------------------------------------------------------------------------------
     Press 'q' to quit, Space to select/deselect
     -------------------------------------------------------------------------------------------------------- */

    //Setup for terminal backend
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    loop {
        // Draw the UI
        terminal.draw(|f| {
            let size = f.size(); // Get the terminal size

            // Vertical Layout for downloads and info
            let vertical_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(size);

            // Horizontal Layout for downloads and main area
            let horizontal_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(40), Constraint::Min(1)])
                .split(vertical_layout[0]);

            // Downloads block
            let downloads_block = Block::default().borders(Borders::ALL).title("Downloads");
            f.render_widget(downloads_block.clone(), horizontal_layout[0]);
            let downloads_inner = downloads_block.inner(horizontal_layout[0]);

            let items: Vec<ListItem> = downloads
                .iter()
                .map(|d| {
                    let bar = progress_bar(d.progress, 14);
                    let display_name = if d.name.len() > 20 {
                        format!("{}..", &d.name[0..14])
                    } else {
                        d.name.clone()
                    };
                    ListItem::new(format!("{} {}", display_name, bar))
                })
                .collect();

            let list = List::new(items).highlight_style(Style::default().bg(Color::Blue));

            // Render the list inside the block
            f.render_stateful_widget(list, downloads_inner, &mut list_state);

            // Main area paragraph widget
            let paragraph = Paragraph::new("test").block(Block::default().borders(Borders::ALL));

            // Instructions paragraph widget
            let instructions = Paragraph::new("Press 'q' to quit, Space to select/deselect")
                .block(Block::default().borders(Borders::ALL).title("Info"));

            // Render widgets
            f.render_widget(paragraph, horizontal_layout[1]);
            f.render_widget(instructions, vertical_layout[1]);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.code == crossterm::event::KeyCode::Char('q') {
                    break; //Quit on 'q'
                } else if key.code == crossterm::event::KeyCode::Up {
                    let i = list_state.selected().unwrap_or(0);
                    list_state.select(Some(i.saturating_sub(1)));
                } else if key.code == crossterm::event::KeyCode::Down {
                    let i = list_state.selected().unwrap_or(0);
                    list_state.select(Some((i + 1).min(downloads.len() - 1)));
                }
            }
        }
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
