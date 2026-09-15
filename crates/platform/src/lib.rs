mod app;

pub use app::run;

// Only what opening a window and clearing it needs. Grows as the next
// steps (input, drawing) land.
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "GameForge".to_string(),
            width: 1280,
            height: 720,
        }
    }
}
