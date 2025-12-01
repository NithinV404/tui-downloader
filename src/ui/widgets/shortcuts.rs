//! Shortcuts guide widget for displaying keyboard shortcuts

use crate::models::InputMode;
use crate::ui::theme::{KeyStyle, Styles};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

/// Render the shortcuts guide widget
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Area to render in
/// * `mode` - Current input mode (determines which shortcuts to show)
pub fn render(f: &mut Frame, area: Rect, mode: InputMode) {
    let shortcuts = get_shortcuts_for_mode(mode);

    let paragraph = Paragraph::new(shortcuts)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("━━ Shortcuts ━━")
                .border_style(Styles::border()),
        )
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Get shortcuts based on input mode
fn get_shortcuts_for_mode(mode: InputMode) -> Vec<Line<'static>> {
    match mode {
        InputMode::Editing => editing_mode_shortcuts(),
        InputMode::Normal => normal_mode_shortcuts(),
    }
}

/// Shortcuts for normal mode
fn normal_mode_shortcuts() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::raw("  "),
            key("[I]"),
            desc(" Add  "),
            key("[Space/P]"),
            desc(" Pause  "),
            key("[D]"),
            desc(" Delete  "),
            key("[Shift+Del]"),
            desc(" Delete File  "),
        ]),
        Line::from(vec![
            Span::raw("  "),
            key("[X]"),
            desc(" Purge  "),
            key("[1/2/3]"),
            desc(" Tabs  "),
            key("[↑↓/jk]"),
            desc(" Move  "),
            key("[Q]"),
            desc(" Quit"),
        ]),
    ]
}

/// Shortcuts for editing mode
fn editing_mode_shortcuts() -> Vec<Line<'static>> {
    vec![Line::from(vec![
        Span::raw("  "),
        key("[Enter]"),
        desc(" Submit  "),
        key("[Esc]"),
        desc(" Cancel  "),
        key("[Backspace]"),
        desc(" Delete  "),
        key("[Ctrl+V]"),
        desc(" Paste"),
    ])]
}

/// Create a styled key span
fn key(text: &'static str) -> Span<'static> {
    Span::styled(text, KeyStyle::key())
}

/// Create a styled description span
fn desc(text: &'static str) -> Span<'static> {
    Span::styled(text, KeyStyle::description())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_mode_has_shortcuts() {
        let shortcuts = normal_mode_shortcuts();
        assert!(!shortcuts.is_empty());
    }

    #[test]
    fn test_editing_mode_has_shortcuts() {
        let shortcuts = editing_mode_shortcuts();
        assert!(!shortcuts.is_empty());
    }

    #[test]
    fn test_mode_switching() {
        let normal = get_shortcuts_for_mode(InputMode::Normal);
        let editing = get_shortcuts_for_mode(InputMode::Editing);

        // They should be different
        assert_ne!(normal.len(), 0);
        assert_ne!(editing.len(), 0);
    }
}
