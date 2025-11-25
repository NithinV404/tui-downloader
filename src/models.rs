/// Represents a download item with metadata
#[derive(Clone, Debug)]
pub struct Download {
    pub name: String,
    pub progress: f64,
    pub speed: String,
    pub status: String,
}

/// Input mode for the application
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum InputMode {
    Normal,
    Editing,
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Normal
    }
}
