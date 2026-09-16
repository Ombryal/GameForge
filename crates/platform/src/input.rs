//! Tracks which keys are currently held down.

use std::collections::HashSet;
use winit::keyboard::KeyCode;

/// Snapshot of which physical keys are currently held.
///
/// Uses physical keycodes rather than logical characters, so WASD-style
/// controls land on the same physical keys regardless of the user's
/// keyboard layout.
#[derive(Default)]
pub struct InputState {
    held: HashSet<KeyCode>,
}

impl InputState {
    /// Whether `key` is currently held down.
    pub fn is_held(&self, key: KeyCode) -> bool {
        self.held.contains(&key)
    }

    pub(crate) fn set(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            self.held.insert(key);
        } else {
            self.held.remove(&key);
        }
    }
}
