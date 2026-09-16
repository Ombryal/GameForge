// Manual smoke test — opens a real OS window, so there's no headless way
// to verify it automatically. `cargo run --example open_window -p
// gameforge-platform` and confirm a window opens at 1280x720, shows dark
// gray, and closes cleanly on the close button.
use gameforge_platform::{Frame, InputState, WindowConfig};

struct Blank;

impl Frame for Blank {
    fn update(&mut self, _dt: f32, _input: &InputState) {}

    fn render(&mut self, pixels: &mut [u32], _width: u32, _height: u32) {
        pixels.fill(0xff181818);
    }
}

fn main() {
    gameforge_platform::run(WindowConfig::default(), Blank);
}
