use crate::models::{Download, InputMode};

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, LineGauge, ListState, Paragraph, Tabs},
};

const APP_VERSION: &str = "1.0";

pub fn render_input_field(f: &mut Frame, area: Rect, input_text: &str, input_mode: InputMode) {
    let input_field = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL).title("Input URL"))
        .style(Style::default().fg(if input_mode == InputMode::Editing {
            Color::Yellow
        } else {
            Color::White
        }));

    f.render_widget(input_field, area)
}

pub fn render_tabs(f: &mut Frame, area: Rect, current_tab: usize, tab_titles: Vec<&str>) {
    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("TUI Downloader {}", APP_VERSION)),
        )
        .select(current_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow));

    f.render_widget(tabs, area)
}

pub fn render_downloads_list(
    f: &mut Frame,
    area: Rect,
    list: &Vec<&Download>,
    list_state: &mut ListState,
) {
    let mut scroll_offset: usize = 0;

    let selected_index = list_state.selected().unwrap_or(0);
    let list_area = area;

    if list_state.selected().unwrap_or(0) >= list.len() {
        list_state.select(Some(list.len().saturating_sub(1)));
    }

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

    for (idx, item) in list.iter().enumerate() {
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
        } else if item.progress == 1.0 {
            Style::default().fg(Color::Green)
        } else {
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
            } else if item.progress == 1.0 {
                Color::Green
            } else {
                Color::LightYellow
            }));

        f.render_widget(gauge, item_layout[1]);

        current_y += 2;
    }

    // Render list border
    let list_border = Block::default().borders(Borders::ALL).title(format!(
        "Downloads [{}/{}]",
        selected_index + 1,
        list.len()
    ));
    f.render_widget(list_border, area);
}

pub fn render_details_pane(f: &mut Frame, size: Rect, items: Download) {
    let status_text = format!(
        "STATUS: {}\n\nFile: {}\nSize: 4.7 GB\nServer: mirrors.edge.org\nPath: ~/Downloads/ISOs",
        items.status, items.name
    );
    let logs_text = "> Handshake successful\n> Allocating disk space...\n> Connected to peer 1\n> Connected to peer 2\n> Chunk 1452 verified\n> Rate limit: None";

    let details =
        Paragraph::new(status_text).block(Block::default().borders(Borders::ALL).title("Details"));
    let logs =
        Paragraph::new(logs_text).block(Block::default().borders(Borders::ALL).title("LOGS"));

    // Vertical split for details/logs
    let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(size);

    f.render_widget(details, right_layout[0]);
    f.render_widget(logs, right_layout[1]);
}

pub fn render_input_guide(f: &mut Frame, size: Rect, input_mode: InputMode) {
    // Instructions with better contrast and styling
    let instructions = Paragraph::new(if input_mode == InputMode::Editing {
        "<Esc> Quit  <Backspace> Clear  <Ctrl+Shift+V> Paste"
    } else {
        "<Space> Pause/Resume   <D> Delete   <Enter> Open Details   <Q> Quit  <I> Input Mode  <↑↓> Move Up/Down"
    })
    .style(Style::default().bg(Color::Cyan).fg(Color::Black))
    .alignment(Alignment::Center);

    f.render_widget(instructions, size)
}
