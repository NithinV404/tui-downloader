use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{List, ListItem, Paragraph},
};
use std::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

            // Vertical Layout for side bar
            let vertical_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(size);

            // Horizontal Layout for content and info bottom widget
            let horizontal_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(20), Constraint::Min(1)])
                .split(vertical_layout[0]);

            // Main area paragraph widget
            let paragraph = Paragraph::new("test").block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::LEFT)
                    .title("Main area"),
            );

            // Instructions paragraph widget
            let instructions = Paragraph::new("Press 'q' to quit").block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Info"),
            );

            let items: Vec<ListItem> = vec!["Item 1", "Item 2", "Item 3"]
                .into_iter()
                .map(ListItem::new)
                .collect();

            let list = List::new(items).block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Downloads"),
            );

            // Render it to fill the screen
            f.render_widget(paragraph, horizontal_layout[1]);
            f.render_widget(list, vertical_layout[0]);
            f.render_widget(instructions, vertical_layout[1]);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.code == crossterm::event::KeyCode::Char('q') {
                    break; //Quit on 'q'
                }
            }
        }
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
