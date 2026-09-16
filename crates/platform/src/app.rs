use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Instant;

use gameforge_runtime::FixedTimestep;
use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

use crate::{InputState, WindowConfig};

/// Fixed simulation step, independent of display refresh rate.
const FIXED_DT: f32 = 1.0 / 60.0;

/// Upper bound on how much wall-clock time a single frame feeds into the
/// accumulator. Without this, a long stall (window drag, breakpoint, OS
/// scheduling hiccup) queues up hundreds of catch-up update() calls in
/// one frame — a slow frame causing the next frame to run even more
/// simulation steps, making it slower still.
const MAX_FRAME_TIME: f32 = 0.25;

/// Per-frame hook into the platform's window and input.
pub trait Frame {
    /// Called once per fixed simulation step, `dt` seconds apart every
    /// time regardless of actual frame rate. Movement, physics, and
    /// anything that should look the same at 30fps and 240fps belongs
    /// here rather than in `render`.
    fn update(&mut self, dt: f32, input: &InputState);

    /// Called once per rendered frame, after whichever `update` calls ran
    /// this frame. May run more or less often than `update` — draw
    /// current state here, don't advance simulation.
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
    timestep: FixedTimestep,
    last_frame: Option<Instant>,
    window: Option<Arc<Window>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
}

impl<G: Frame> App<G> {
    fn new(config: WindowConfig, game: G) -> Self {
        Self {
            config,
            game,
            input: InputState::default(),
            timestep: FixedTimestep::new(FIXED_DT),
            last_frame: None,
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

        let now = Instant::now();
        let dt = self
            .last_frame
            .map_or(0.0, |prev| (now - prev).as_secs_f32())
            .min(MAX_FRAME_TIME);
        self.last_frame = Some(now);

        let steps = self.timestep.advance(dt);
        for _ in 0..steps {
            self.game.update(self.timestep.step(), &self.input);
        }

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
