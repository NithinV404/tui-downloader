use crate::models::InputMode;
use crossterm::event::KeyModifiers;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum KeyAction {
    // Normal mode actions
    EnterEditMode,
    Quit,
    SelectTab(usize),
    MoveUp,
    MoveDown,
    PauseResume,
    Delete,
    DeleteFile,
    PurgeCompleted,

    // New navigation actions
    MoveToTop,
    MoveToBottom,
    PageUp,
    PageDown,

    // Search and filter
    EnterSearchMode,
    ClearSearch,

    // Help
    ShowHelp,

    // Speed limit
    ShowSpeedLimit,

    // Retry failed download
    RetryDownload,

    // Open file/folder
    OpenFile,
    OpenFolder,

    // Copy to clipboard
    CopyUrl,
    CopyPath,

    // Sorting
    CycleSort,
    ToggleSortDirection,

    // Queue management
    MoveQueueUp,
    MoveQueueDown,

    // Batch operations
    ToggleSelect,
    SelectAll,
    DeselectAll,

    // Pause/Resume all
    PauseAll,
    ResumeAll,

    // Editing mode actions
    SubmitInput,
    CancelInput,
    DeleteChar,
    DeleteWord,
    ClearAll,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorStart,
    MoveCursorEnd,

    // Search mode actions
    SearchSubmit,
    SearchCancel,
    SearchDeleteChar,

    // Speed limit mode actions
    SpeedLimitConfirm,
    SpeedLimitCancel,
    SpeedLimitToggleField,
    SpeedLimitIncrease,
    SpeedLimitDecrease,

    // Help mode actions
    HelpClose,
    HelpScrollUp,
    HelpScrollDown,

    // Confirmation actions
    ConfirmYes,
    ConfirmNo,

    // No action
    None,
}

pub struct InputHandler {
    pub mode: InputMode,
    pub buffer: String,
    pub search_query: String,
    pub cursor_position: usize,
    pub speed_limit_buffer: String,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            mode: InputMode::Normal,
            buffer: String::new(),
            search_query: String::new(),
            cursor_position: 0,
            speed_limit_buffer: String::new(),
        }
    }

    pub fn handle_key(&mut self, key: &crossterm::event::KeyEvent) -> KeyAction {
        match self.mode {
            InputMode::Normal => self.handle_normal_mode(key),
            InputMode::Editing => self.handle_input_mode(key),
            InputMode::Search => self.handle_search_mode(key),
            InputMode::SpeedLimit => self.handle_speed_limit_mode(key),
            InputMode::Help => self.handle_help_mode(key),
            InputMode::Confirmation => self.handle_confirmation_mode(key),
            InputMode::Settings => self.handle_settings_mode(key),
        }
    }

    pub fn handle_normal_mode(&mut self, key: &crossterm::event::KeyEvent) -> KeyAction {
        use crossterm::event::KeyCode;

        // Check for modifier combinations first
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            match key.code {
                KeyCode::Delete => return KeyAction::DeleteFile,
                KeyCode::Up | KeyCode::Char('K') => return KeyAction::MoveQueueUp,
                KeyCode::Down | KeyCode::Char('J') => return KeyAction::MoveQueueDown,
                KeyCode::Char('P') => return KeyAction::PauseAll,
                KeyCode::Char('R') => return KeyAction::ResumeAll,
                _ => {}
            }
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('a') => return KeyAction::SelectAll,
                KeyCode::Char('d') => return KeyAction::DeselectAll,
                KeyCode::Char('u') => return KeyAction::PageUp,
                _ => {}
            }
        }

        match key.code {
            // Basic actions
            KeyCode::Char('i') | KeyCode::Char('I') => KeyAction::EnterEditMode,
            KeyCode::Char('q') | KeyCode::Char('Q') => KeyAction::Quit,

            // Tab selection
            KeyCode::Char('1') => KeyAction::SelectTab(0),
            KeyCode::Char('2') => KeyAction::SelectTab(1),
            KeyCode::Char('3') => KeyAction::SelectTab(2),

            // Navigation
            KeyCode::Up | KeyCode::Char('k') => KeyAction::MoveUp,
            KeyCode::Down | KeyCode::Char('j') => KeyAction::MoveDown,
            KeyCode::Home | KeyCode::Char('g') => KeyAction::MoveToTop,
            KeyCode::End | KeyCode::Char('G') => KeyAction::MoveToBottom,
            KeyCode::PageUp => KeyAction::PageUp,
            KeyCode::PageDown => KeyAction::PageDown,

            // Download management
            KeyCode::Char(' ') | KeyCode::Char('p') => KeyAction::PauseResume,
            KeyCode::Char('d') => KeyAction::Delete,
            KeyCode::Char('x') | KeyCode::Char('X') => KeyAction::PurgeCompleted,
            KeyCode::Char('r') => KeyAction::RetryDownload,

            // Search
            KeyCode::Char('/') => KeyAction::EnterSearchMode,
            KeyCode::Esc => KeyAction::ClearSearch,

            // Help
            KeyCode::Char('?') => KeyAction::ShowHelp,
            KeyCode::F(1) => KeyAction::ShowHelp,

            // Speed limit
            KeyCode::Char('l') | KeyCode::Char('L') => KeyAction::ShowSpeedLimit,

            // Open file/folder
            KeyCode::Char('o') => KeyAction::OpenFile,
            KeyCode::Char('O') => KeyAction::OpenFolder,

            // Copy
            KeyCode::Char('c') => KeyAction::CopyUrl,
            KeyCode::Char('C') => KeyAction::CopyPath,

            // Sorting
            KeyCode::Char('s') => KeyAction::CycleSort,
            KeyCode::Char('S') => KeyAction::ToggleSortDirection,

            // Selection
            KeyCode::Char('v') | KeyCode::Char('V') => KeyAction::ToggleSelect,

            _ => KeyAction::None,
        }
    }

    pub fn handle_input_mode(&mut self, key: &crossterm::event::KeyEvent) -> KeyAction {
        use crossterm::event::KeyCode;

        // Handle Ctrl combinations
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('u') => {
                    self.buffer.clear();
                    self.cursor_position = 0;
                    return KeyAction::ClearAll;
                }
                KeyCode::Char('w') => {
                    // Delete word backwards
                    if self.cursor_position > 0 {
                        let before_cursor = &self.buffer[..self.cursor_position];
                        let trimmed = before_cursor.trim_end();
                        let last_space = trimmed.rfind(' ').map(|i| i + 1).unwrap_or(0);
                        let after_cursor = &self.buffer[self.cursor_position..];
                        self.buffer = format!("{}{}", &self.buffer[..last_space], after_cursor);
                        self.cursor_position = last_space;
                    }
                    return KeyAction::DeleteWord;
                }
                KeyCode::Char('a') => {
                    self.cursor_position = 0;
                    return KeyAction::MoveCursorStart;
                }
                KeyCode::Char('e') => {
                    self.cursor_position = self.buffer.len();
                    return KeyAction::MoveCursorEnd;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Enter => KeyAction::SubmitInput,
            KeyCode::Esc => KeyAction::CancelInput,
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.buffer.remove(self.cursor_position);
                }
                KeyAction::DeleteChar
            }
            KeyCode::Delete => {
                if self.cursor_position < self.buffer.len() {
                    self.buffer.remove(self.cursor_position);
                }
                KeyAction::DeleteChar
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                KeyAction::MoveCursorLeft
            }
            KeyCode::Right => {
                if self.cursor_position < self.buffer.len() {
                    self.cursor_position += 1;
                }
                KeyAction::MoveCursorRight
            }
            KeyCode::Home => {
                self.cursor_position = 0;
                KeyAction::MoveCursorStart
            }
            KeyCode::End => {
                self.cursor_position = self.buffer.len();
                KeyAction::MoveCursorEnd
            }
            KeyCode::Char(c) => {
                self.buffer.insert(self.cursor_position, c);
                self.cursor_position += 1;
                KeyAction::None
            }
            _ => KeyAction::None,
        }
    }

    pub fn handle_search_mode(&mut self, key: &crossterm::event::KeyEvent) -> KeyAction {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Enter => {
                self.mode = InputMode::Normal;
                KeyAction::SearchSubmit
            }
            KeyCode::Esc => {
                self.search_query.clear();
                self.mode = InputMode::Normal;
                KeyAction::SearchCancel
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                KeyAction::SearchDeleteChar
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                KeyAction::None
            }
            _ => KeyAction::None,
        }
    }

    pub fn handle_speed_limit_mode(&mut self, key: &crossterm::event::KeyEvent) -> KeyAction {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Enter => KeyAction::SpeedLimitConfirm,
            KeyCode::Esc => KeyAction::SpeedLimitCancel,
            KeyCode::Tab | KeyCode::Up | KeyCode::Down => KeyAction::SpeedLimitToggleField,
            KeyCode::Right => KeyAction::SpeedLimitIncrease,
            KeyCode::Left => KeyAction::SpeedLimitDecrease,
            KeyCode::Backspace => {
                self.speed_limit_buffer.pop();
                KeyAction::None
            }
            KeyCode::Char(c)
                if c.is_ascii_digit() || c == '.' || c == 'm' || c == 'k' || c == 'g' =>
            {
                self.speed_limit_buffer.push(c);
                KeyAction::None
            }
            _ => KeyAction::None,
        }
    }

    pub fn handle_help_mode(&mut self, key: &crossterm::event::KeyEvent) -> KeyAction {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Enter => {
                KeyAction::HelpClose
            }
            KeyCode::Up | KeyCode::Char('k') => KeyAction::HelpScrollUp,
            KeyCode::Down | KeyCode::Char('j') => KeyAction::HelpScrollDown,
            _ => KeyAction::None,
        }
    }

    pub fn handle_confirmation_mode(&mut self, key: &crossterm::event::KeyEvent) -> KeyAction {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => KeyAction::ConfirmYes,
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => KeyAction::ConfirmNo,
            _ => KeyAction::None,
        }
    }

    pub fn handle_settings_mode(&mut self, key: &crossterm::event::KeyEvent) -> KeyAction {
        use crossterm::event::KeyCode;

        // Settings mode shares similar behavior to confirmation for now
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => KeyAction::CancelInput,
            KeyCode::Enter => KeyAction::SubmitInput,
            _ => KeyAction::None,
        }
    }

    pub fn handle_paste(&mut self, data: &str) {
        match self.mode {
            InputMode::Editing => {
                self.buffer.insert_str(self.cursor_position, data);
                self.cursor_position += data.len();
            }
            InputMode::Search => {
                self.search_query.push_str(data);
            }
            InputMode::SpeedLimit => {
                // Only allow numeric pastes for speed limit
                let cleaned: String = data
                    .chars()
                    .filter(|c| {
                        c.is_ascii_digit() || *c == '.' || *c == 'm' || *c == 'k' || *c == 'g'
                    })
                    .collect();
                self.speed_limit_buffer.push_str(&cleaned);
            }
            _ => {}
        }
    }

    pub fn enter_edit_mode(&mut self) {
        self.mode = InputMode::Editing;
        self.buffer.clear();
        self.cursor_position = 0;
    }

    pub fn enter_search_mode(&mut self) {
        self.mode = InputMode::Search;
        self.search_query.clear();
    }

    pub fn enter_speed_limit_mode(&mut self) {
        self.mode = InputMode::SpeedLimit;
        self.speed_limit_buffer.clear();
    }

    pub fn enter_help_mode(&mut self) {
        self.mode = InputMode::Help;
    }

    pub fn enter_confirmation_mode(&mut self) {
        self.mode = InputMode::Confirmation;
    }

    pub fn exit_edit_mode(&mut self) {
        self.mode = InputMode::Normal;
    }

    pub fn exit_to_normal(&mut self) {
        self.mode = InputMode::Normal;
    }

    #[allow(dead_code)]
    pub fn delete_last_char(&mut self) {
        match self.mode {
            InputMode::Editing => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.buffer.remove(self.cursor_position);
                }
            }
            InputMode::Search => {
                self.search_query.pop();
            }
            InputMode::SpeedLimit => {
                self.speed_limit_buffer.pop();
            }
            _ => {}
        }
    }

    pub fn get_input(&self) -> &str {
        &self.buffer
    }

    pub fn get_search_query(&self) -> &str {
        &self.search_query
    }

    #[allow(dead_code)]
    pub fn get_speed_limit_buffer(&self) -> &str {
        &self.speed_limit_buffer
    }

    pub fn take_input(&mut self) -> String {
        self.cursor_position = 0;
        std::mem::take(&mut self.buffer)
    }

    #[allow(dead_code)]
    pub fn take_search_query(&mut self) -> String {
        std::mem::take(&mut self.search_query)
    }

    #[allow(dead_code)]
    pub fn take_speed_limit_buffer(&mut self) -> String {
        std::mem::take(&mut self.speed_limit_buffer)
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
    }

    #[allow(dead_code)]
    pub fn set_buffer(&mut self, text: &str) {
        self.buffer = text.to_string();
        self.cursor_position = self.buffer.len();
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState};

    fn make_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    fn make_key_event_with_mod(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn test_new_handler() {
        let handler = InputHandler::new();
        assert_eq!(handler.mode, InputMode::Normal);
        assert!(handler.buffer.is_empty());
        assert!(handler.search_query.is_empty());
    }

    #[test]
    fn test_enter_edit_mode() {
        let mut handler = InputHandler::new();
        handler.enter_edit_mode();
        assert_eq!(handler.mode, InputMode::Editing);
    }

    #[test]
    fn test_enter_search_mode() {
        let mut handler = InputHandler::new();
        handler.enter_search_mode();
        assert_eq!(handler.mode, InputMode::Search);
    }

    #[test]
    fn test_handle_paste() {
        let mut handler = InputHandler::new();
        handler.enter_edit_mode();
        handler.handle_paste("https://example.com");
        assert_eq!(handler.buffer, "https://example.com");
        assert_eq!(handler.cursor_position, 19);
    }

    #[test]
    fn test_cursor_movement() {
        let mut handler = InputHandler::new();
        handler.enter_edit_mode();
        handler.buffer = "test".to_string();
        handler.cursor_position = 4;

        handler.handle_key(&make_key_event(KeyCode::Left));
        assert_eq!(handler.cursor_position, 3);

        handler.handle_key(&make_key_event(KeyCode::Home));
        assert_eq!(handler.cursor_position, 0);

        handler.handle_key(&make_key_event(KeyCode::End));
        assert_eq!(handler.cursor_position, 4);
    }

    #[test]
    fn test_normal_mode_keys() {
        let mut handler = InputHandler::new();

        let action = handler.handle_key(&make_key_event(KeyCode::Char('?')));
        assert!(matches!(action, KeyAction::ShowHelp));

        let action = handler.handle_key(&make_key_event(KeyCode::Char('/')));
        assert!(matches!(action, KeyAction::EnterSearchMode));

        let action = handler.handle_key(&make_key_event(KeyCode::Char('l')));
        assert!(matches!(action, KeyAction::ShowSpeedLimit));
    }

    #[test]
    fn test_shift_delete() {
        let mut handler = InputHandler::new();
        let action = handler.handle_key(&make_key_event_with_mod(
            KeyCode::Delete,
            KeyModifiers::SHIFT,
        ));
        assert!(matches!(action, KeyAction::DeleteFile));
    }

    #[test]
    fn test_search_mode() {
        let mut handler = InputHandler::new();
        handler.enter_search_mode();

        handler.handle_key(&make_key_event(KeyCode::Char('t')));
        handler.handle_key(&make_key_event(KeyCode::Char('e')));
        handler.handle_key(&make_key_event(KeyCode::Char('s')));
        handler.handle_key(&make_key_event(KeyCode::Char('t')));

        assert_eq!(handler.search_query, "test");

        handler.handle_key(&make_key_event(KeyCode::Backspace));
        assert_eq!(handler.search_query, "tes");
    }

    #[test]
    fn test_take_input() {
        let mut handler = InputHandler::new();
        handler.buffer = "test".to_string();
        let taken = handler.take_input();
        assert_eq!(taken, "test");
        assert!(handler.buffer.is_empty());
    }

    #[test]
    fn test_clear_search() {
        let mut handler = InputHandler::new();
        handler.search_query = "query".to_string();
        handler.clear_search();
        assert!(handler.search_query.is_empty());
    }
}
