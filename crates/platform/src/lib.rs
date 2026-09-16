//! Windowing, input, and the raw pixel surface games draw into.
//!
//! Doesn't know about entities, components, or game-specific rendering —
//! that's `renderer2d` and later crates. This crate's job stops at "here's
//! a window, here's what keys are held, here's a buffer to write pixels
//! into."

mod app;
mod input;

pub use app::{run, Frame};
pub use input::InputState;

/// Configuration for the window this crate opens.
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
