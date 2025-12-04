//! Shortcuts guide widget for displaying keyboard shortcuts

#![allow(dead_code)]

use crate::models::InputMode;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render the shortcuts guide widget
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Area to render in
/// * `mode` - Current input mode (determines which shortcuts to show)
pub fn render(f: &mut Frame, area: Rect, mode: InputMode) {
    render_with_search(f, area, mode, false)
}

/// Render the shortcuts guide widget with optional search indicator
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Area to render in
/// * `mode` - Current input mode (determines which shortcuts to show)
/// * `has_search` - Whether a search filter is active
pub fn render_with_search(f: &mut Frame, area: Rect, mode: InputMode, has_search: bool) {
    let shortcuts = get_shortcuts_for_mode(mode, has_search);

    let paragraph = Paragraph::new(shortcuts).alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

/// Get shortcuts based on input mode
fn get_shortcuts_for_mode(mode: InputMode, has_search: bool) -> Vec<Line<'static>> {
    match mode {
        InputMode::Editing => editing_mode_shortcuts(),
        InputMode::Search => search_mode_shortcuts(),
        InputMode::SpeedLimit => speed_limit_mode_shortcuts(),
        InputMode::Help => help_mode_shortcuts(),
        InputMode::Confirmation => confirmation_mode_shortcuts(),
        InputMode::Settings => settings_mode_shortcuts(),
        InputMode::Normal => normal_mode_shortcuts(has_search),
    }
}

/// Shortcuts for normal mode
fn normal_mode_shortcuts(has_search: bool) -> Vec<Line<'static>> {
    if has_search {
        vec![
            Line::from(vec![
                key("i"),
                desc(" add   "),
                key("/"),
                desc(" search   "),
                key("Esc"),
                desc(" clear   "),
                key("j/k"),
                desc(" move   "),
                key("Space"),
                desc(" pause   "),
                key("?"),
                desc(" help"),
            ]),
            Line::from(vec![
                key("d"),
                desc(" delete   "),
                key("r"),
                desc(" retry   "),
                key("s"),
                desc(" sort   "),
                key("l"),
                desc(" limits   "),
                key("q"),
                desc(" quit"),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                key("i"),
                desc(" add   "),
                key("/"),
                desc(" search   "),
                key("Space"),
                desc(" pause   "),
                key("d"),
                desc(" delete   "),
                key("?"),
                desc(" help"),
            ]),
            Line::from(vec![
                key("r"),
                desc(" retry   "),
                key("s"),
                desc(" sort   "),
                key("l"),
                desc(" limits   "),
                key("o"),
                desc(" open   "),
                key("1-3"),
                desc(" tabs   "),
                key("q"),
                desc(" quit"),
            ]),
        ]
    }
}

/// Shortcuts for editing mode
fn editing_mode_shortcuts() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            key("Enter"),
            desc(" submit   "),
            key("Esc"),
            desc(" cancel   "),
            key("Ctrl+U"),
            desc(" clear   "),
            key("Ctrl+W"),
            desc(" del word"),
        ]),
        Line::from(vec![
            key("<- ->"),
            desc(" move   "),
            key("Home/End"),
            desc(" start/end   "),
            key("Backspace"),
            desc(" delete   "),
            key("Ctrl+V"),
            desc(" paste"),
        ]),
    ]
}

/// Shortcuts for search mode
fn search_mode_shortcuts() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("[/] ", Style::default().fg(Theme::HIGHLIGHT)),
            desc("Type to filter   "),
            key("Enter"),
            desc(" apply   "),
            key("Esc"),
            desc(" clear & exit   "),
            key("Backspace"),
            desc(" delete"),
        ]),
        Line::from(vec![]),
    ]
}

/// Shortcuts for speed limit mode
fn speed_limit_mode_shortcuts() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("[!] ", Style::default().fg(Theme::WARNING)),
            desc("Set bandwidth   "),
            key("Tab"),
            desc(" switch DL/UL   "),
            key("j/k"),
            desc(" adjust   "),
            key("Enter"),
            desc(" apply   "),
            key("Esc"),
            desc(" cancel"),
        ]),
        Line::from(vec![]),
    ]
}

/// Shortcuts for help mode
fn help_mode_shortcuts() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("[?] ", Style::default().fg(Theme::INFO)),
            desc("Viewing help   "),
            key("j/k"),
            desc(" scroll   "),
            key("Esc/?/q/Enter"),
            desc(" close"),
        ]),
        Line::from(vec![]),
    ]
}

/// Shortcuts for confirmation mode
fn confirmation_mode_shortcuts() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("[!] ", Style::default().fg(Theme::WARNING)),
            desc("Confirm action   "),
            key("y"),
            desc(" yes   "),
            key("n/Esc"),
            desc(" no"),
        ]),
        Line::from(vec![]),
    ]
}

/// Shortcuts for settings mode
fn settings_mode_shortcuts() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("[=] ", Style::default().fg(Theme::INFO)),
            desc("Settings   "),
            key("j/k"),
            desc(" navigate   "),
            key("Enter"),
            desc(" edit   "),
            key("Esc"),
            desc(" close"),
        ]),
        Line::from(vec![]),
    ]
}

/// Create a styled key span
fn key(text: &'static str) -> Span<'static> {
    Span::styled(
        text,
        Style::default()
            .fg(Theme::SECONDARY)
            .add_modifier(Modifier::BOLD),
    )
}

/// Create a styled description span
fn desc(text: &'static str) -> Span<'static> {
    Span::styled(text, Style::default().fg(Theme::TEXT_MUTED))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_mode_has_shortcuts() {
        let shortcuts = normal_mode_shortcuts(false);
        assert!(!shortcuts.is_empty());
        assert_eq!(shortcuts.len(), 2); // Two rows of shortcuts
    }

    #[test]
    fn test_normal_mode_with_search() {
        let shortcuts = normal_mode_shortcuts(true);
        assert!(!shortcuts.is_empty());
        assert_eq!(shortcuts.len(), 2);
    }

    #[test]
    fn test_editing_mode_has_shortcuts() {
        let shortcuts = editing_mode_shortcuts();
        assert!(!shortcuts.is_empty());
        assert_eq!(shortcuts.len(), 2);
    }

    #[test]
    fn test_search_mode_has_shortcuts() {
        let shortcuts = search_mode_shortcuts();
        assert!(!shortcuts.is_empty());
        assert_eq!(shortcuts.len(), 2);
    }

    #[test]
    fn test_speed_limit_mode_has_shortcuts() {
        let shortcuts = speed_limit_mode_shortcuts();
        assert!(!shortcuts.is_empty());
    }

    #[test]
    fn test_help_mode_has_shortcuts() {
        let shortcuts = help_mode_shortcuts();
        assert!(!shortcuts.is_empty());
    }

    #[test]
    fn test_confirmation_mode_has_shortcuts() {
        let shortcuts = confirmation_mode_shortcuts();
        assert!(!shortcuts.is_empty());
    }

    #[test]
    fn test_settings_mode_has_shortcuts() {
        let shortcuts = settings_mode_shortcuts();
        assert!(!shortcuts.is_empty());
    }

    #[test]
    fn test_mode_switching() {
        let normal = get_shortcuts_for_mode(InputMode::Normal, false);
        let editing = get_shortcuts_for_mode(InputMode::Editing, false);
        let search = get_shortcuts_for_mode(InputMode::Search, false);

        // They should all have content
        assert!(!normal.is_empty());
        assert!(!editing.is_empty());
        assert!(!search.is_empty());
    }

    #[test]
    fn test_all_modes_have_content() {
        let modes = vec![
            InputMode::Normal,
            InputMode::Editing,
            InputMode::Search,
            InputMode::SpeedLimit,
            InputMode::Help,
            InputMode::Confirmation,
            InputMode::Settings,
        ];

        for mode in modes {
            let shortcuts = get_shortcuts_for_mode(mode, false);
            assert!(
                !shortcuts.is_empty(),
                "Mode {:?} should have shortcuts",
                mode
            );
        }
    }
}
