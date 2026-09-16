// Manual smoke test for input, timing, and drawing together. Run with:
//   cargo run --example move_rect -p gameforge-platform
// WASD or arrow keys should move a blue square at a constant speed
// regardless of frame rate — try resizing the window while holding a key
// to confirm speed doesn't change.
use gameforge_math::Vec2;
use gameforge_platform::{Frame, InputState, WindowConfig};
use gameforge_renderer2d::Canvas;
use winit::keyboard::KeyCode;

const RECT_SIZE: u32 = 40;
const MOVE_SPEED: f32 = 240.0; // pixels per second

struct MoveRect {
    position: Vec2,
}

impl Frame for MoveRect {
    fn update(&mut self, dt: f32, input: &InputState) {
        let mut dir = Vec2::zero();
        if input.is_held(KeyCode::KeyW) || input.is_held(KeyCode::ArrowUp) {
            dir.y -= 1.0;
        }
        if input.is_held(KeyCode::KeyS) || input.is_held(KeyCode::ArrowDown) {
            dir.y += 1.0;
        }
        if input.is_held(KeyCode::KeyA) || input.is_held(KeyCode::ArrowLeft) {
            dir.x -= 1.0;
        }
        if input.is_held(KeyCode::KeyD) || input.is_held(KeyCode::ArrowRight) {
            dir.x += 1.0;
        }

        // normalized() keeps diagonal movement from outrunning a single
        // axis — without it, W+D together would move at sqrt(2) times
        // the intended speed instead of matching it.
        let step = dir.normalized().scale(MOVE_SPEED * dt);
        self.position = self.position.add(step);
    }

    fn render(&mut self, pixels: &mut [u32], width: u32, height: u32) {
        let mut canvas = Canvas::new(pixels, width, height);
        canvas.clear(0xff181818);
        canvas.fill_rect(
            self.position.x as i32,
            self.position.y as i32,
            RECT_SIZE,
            RECT_SIZE,
            0xff3388ff,
        );
    }
}

fn main() {
    let game = MoveRect { position: Vec2::new(100.0, 100.0) };
    gameforge_platform::run(WindowConfig::default(), game);
}
