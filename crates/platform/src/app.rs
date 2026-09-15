use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

use crate::{InputState, WindowConfig};

/// Per-frame hook into the platform's window and input.
///
/// Deliberately doesn't take a delta time — this layer only owns the
/// window, event loop, and raw pixel buffer. Timing is `runtime`'s job;
/// wiring the two together is a later integration step, not this one.
pub trait Frame {
    /// Called once per frame before `render`, with the latest input snapshot.
    fn update(&mut self, input: &InputState);

    /// Called once per frame to draw. `pixels` is ARGB8888, row-major,
    /// `width * height` elements — hand it to `renderer2d::Canvas` or
    /// write into it directly.
    fn render(&mut self, pixels: &mut [u32], width: u32, height: u32);
}

/// softbuffer's Context/Surface borrow from whatever owns the window
/// handle, so the window has to outlive both. Arc<Window> lets winit's
/// Window and softbuffer's Surface each hold a reference without fighting
/// over ownership.
struct App<G: Frame> {
    config: WindowConfig,
    game: G,
    input: InputState,
    window: Option<Arc<Window>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
}

impl<G: Frame> App<G> {
    fn new(config: WindowConfig, game: G) -> Self {
        Self {
            config,
            game,
            input: InputState::default(),
            window: None,
            surface: None,
        }
    }

    fn draw_frame(&mut self) {
        let (Some(window), Some(surface)) = (&self.window, &mut self.surface) else {
            return;
        };
        let size = window.inner_size();
        let (Some(width), Some(height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            return;
        };

        surface
            .resize(width, height)
            .expect("failed to resize softbuffer surface");

        self.game.update(&self.input);

        let mut buffer = surface
            .buffer_mut()
            .expect("failed to get softbuffer buffer");
        self.game.render(&mut buffer, width.get(), height.get());
        buffer.present().expect("failed to present softbuffer buffer");
    }
}

impl<G: Frame> ApplicationHandler for App<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = Window::default_attributes()
            .with_title(self.config.title.clone())
            .with_inner_size(LogicalSize::new(self.config.width, self.config.height));

        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("failed to create window"),
        );

        let context = Context::new(window.clone()).expect("failed to create softbuffer context");
        let surface =
            Surface::new(&context, window.clone()).expect("failed to create softbuffer surface");

        self.window = Some(window);
        self.surface = Some(surface);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.draw_frame(),
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    let pressed = matches!(event.state, ElementState::Pressed);
                    self.input.set(code, pressed);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // No frame pacing yet — every loop iteration asks for a redraw,
        // which currently runs as fast as the OS schedules it. Frame
        // rate limiting and vsync-aware pacing belong to the runtime
        // integration, not here.
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/// Opens a window and runs `game` until it's closed.
///
/// Blocks the calling thread — winit owns the event loop for the life of
/// the program, so this is meant to be the last call in `main()`.
pub fn run<G: Frame>(config: WindowConfig, game: G) {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(config, game);
    event_loop
        .run_app(&mut app)
        .expect("event loop exited with an error");
}
