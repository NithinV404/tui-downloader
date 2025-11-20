use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, ListState, Paragraph},
};
use std::io;

#[derive(Clone)]
struct Download {
    name: String,
    progress: f64, // 0.0 to 1.0
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    let downloads = vec![
        Download {
            name: "This is a very big test item name testing".to_string(),
            progress: 0.3,
        },
        Download {
            name: "Item 2".to_string(),
            progress: 1.0,
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

            // Layout for each download item (2 lines: name + progress)
            let item_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(4); downloads.len()])
                .split(downloads_inner);

            for (i, item) in downloads.iter().enumerate() {
                let selected = list_state.selected().unwrap_or(0) == i;
                let item_style = if selected {
                    Style::default().bg(Color::Blue)
                } else {
                    Style::default()
                };

                let item_name_len = item.name.len();
                let display_name = if item_name_len > 38 {
                    format!(
                        "{}..{}",
                        &item.name[0..10],
                        &item.name[item_name_len - 10..item_name_len]
                    ) // Truncate to fit
                } else {
                    item.name.clone()
                };

                // Combine name and bar in a single Paragraph with newlines
                let item_text = format!("{}\n ", display_name); // Name on first line, space for gauge below
                let item_para = Paragraph::new(item_text);

                let item_block = Block::default()
                    .borders(Borders::ALL)
                    .style(item_style)
                    .title(format!("{}", i + 1));
                let item_area = item_layout[i];
                f.render_widget(item_block.clone(), item_area);
                let item_inner = item_block.inner(item_area);

                // Sub-layout for name and gauge
                let sub_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Length(1)])
                    .split(item_inner);

                f.render_widget(item_para, sub_layout[0]);

                let gauge = Gauge::default()
                    .block(Block::default()) // Optional: add borders if you want
                    .gauge_style(if (item.progress * 100.0) < 100.0 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Green)
                    })
                    .percent((item.progress * 100.0) as u16)
                    .label(format!("[{:.0}%]", item.progress * 100.0)); // Bracketed percentage

                f.render_widget(gauge, sub_layout[1]);
            }

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
