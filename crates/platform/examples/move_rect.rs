// Manual smoke test for input + drawing together. Run with:
//   cargo run --example move_rect -p gameforge-platform
// WASD or arrow keys should move a blue square around a dark gray window.
use gameforge_platform::{Frame, InputState, WindowConfig};
use gameforge_renderer2d::Canvas;
use winit::keyboard::KeyCode;

const RECT_SIZE: u32 = 40;
const MOVE_SPEED: i32 = 6;

struct MoveRect {
    x: i32,
    y: i32,
}

impl Frame for MoveRect {
    fn update(&mut self, input: &InputState) {
        if input.is_held(KeyCode::KeyW) || input.is_held(KeyCode::ArrowUp) {
            self.y -= MOVE_SPEED;
        }
        if input.is_held(KeyCode::KeyS) || input.is_held(KeyCode::ArrowDown) {
            self.y += MOVE_SPEED;
        }
        if input.is_held(KeyCode::KeyA) || input.is_held(KeyCode::ArrowLeft) {
            self.x -= MOVE_SPEED;
        }
        if input.is_held(KeyCode::KeyD) || input.is_held(KeyCode::ArrowRight) {
            self.x += MOVE_SPEED;
        }
    }

    fn render(&mut self, pixels: &mut [u32], width: u32, height: u32) {
        let mut canvas = Canvas::new(pixels, width, height);
        canvas.clear(0xff181818);
        canvas.fill_rect(self.x, self.y, RECT_SIZE, RECT_SIZE, 0xff3388ff);
    }
}

fn main() {
    let game = MoveRect { x: 100, y: 100 };
    gameforge_platform::run(WindowConfig::default(), game);
}
