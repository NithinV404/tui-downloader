use crate::models::InputMode;
use crossterm::event::KeyModifiers;

#[derive(Debug, Clone, Copy)]
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

    // Editing mode actions
    SubmitInput,
    CancelInput,
    DeleteChar,
    ClearAll,

    // No action
    None,
}

pub struct InputHandler {
    pub mode: InputMode,
    pub buffer: String,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            mode: InputMode::Normal,
            buffer: String::new(),
        }
    }

    pub fn handle_key(&mut self, key: &crossterm::event::KeyEvent) -> KeyAction {
        match self.mode {
            InputMode::Normal => self.handle_normal_mode(key),
            InputMode::Editing => self.handle_input_mode(key),
        }
    }

    pub fn handle_normal_mode(&mut self, key: &crossterm::event::KeyEvent) -> KeyAction {
        use crossterm::event::KeyCode;

        // Check for Shift+Delete to delete file
        if key.code == KeyCode::Delete && key.modifiers.contains(KeyModifiers::SHIFT) {
            return KeyAction::DeleteFile;
        }

        match key.code {
            KeyCode::Char('i') | KeyCode::Char('I') => KeyAction::EnterEditMode,
            KeyCode::Char('q') | KeyCode::Char('Q') => KeyAction::Quit,
            KeyCode::Char('1') => KeyAction::SelectTab(0),
            KeyCode::Char('2') => KeyAction::SelectTab(1),
            KeyCode::Char('3') => KeyAction::SelectTab(2),
            KeyCode::Up | KeyCode::Char('k') => KeyAction::MoveUp,
            KeyCode::Down | KeyCode::Char('j') => KeyAction::MoveDown,
            KeyCode::Char(' ') | KeyCode::Char('p') | KeyCode::Char('P') => KeyAction::PauseResume,
            KeyCode::Char('d') | KeyCode::Char('D') => KeyAction::Delete,
            KeyCode::Char('x') | KeyCode::Char('X') => KeyAction::PurgeCompleted,
            _ => KeyAction::None,
        }
    }

    pub fn handle_input_mode(&mut self, key: &crossterm::event::KeyEvent) -> KeyAction {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Enter => KeyAction::SubmitInput,
            KeyCode::Esc => KeyAction::CancelInput,
            KeyCode::Backspace => KeyAction::DeleteChar,
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.buffer.clear();
                } else {
                    self.buffer.push(c);
                }
                KeyAction::None
            }
            _ => KeyAction::None,
        }
    }

    pub fn handle_paste(&mut self, data: &str) {
        if self.mode == InputMode::Editing {
            self.buffer.push_str(data);
        }
    }

    pub fn enter_edit_mode(&mut self) {
        self.mode = InputMode::Editing;
        self.buffer.clear();
    }

    pub fn exit_edit_mode(&mut self) {
        self.mode = InputMode::Normal;
    }

    pub fn delete_last_char(&mut self) {
        if self.mode == InputMode::Editing {
            self.buffer.pop();
        }
    }

    pub fn get_input(&mut self) -> &str {
        &self.buffer
    }

    pub fn take_input(&mut self) -> String {
        std::mem::take(&mut self.buffer)
    }
}
