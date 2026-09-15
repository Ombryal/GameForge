use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::WindowConfig;

// 0xAARRGGBB — softbuffer's pixel format. Fixed for now; nothing writes
// to this buffer except the clear yet.
const CLEAR_COLOR: u32 = 0xff181818;

// softbuffer's Context and Surface borrow from whatever owns the window
// handle, so the window has to outlive both. Arc<Window> lets winit's
// Window and softbuffer's Surface both hold a reference without fighting
// over ownership.
struct App {
    config: WindowConfig,
    window: Option<Arc<Window>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
}

impl App {
    fn new(config: WindowConfig) -> Self {
        Self { config, window: None, surface: None }
    }

    fn clear(&mut self) {
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
        let mut buffer = surface
            .buffer_mut()
            .expect("failed to get softbuffer buffer");
        buffer.fill(CLEAR_COLOR);
        buffer.present().expect("failed to present softbuffer buffer");
    }
}

impl ApplicationHandler for App {
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
            WindowEvent::RedrawRequested => self.clear(),
            WindowEvent::Resized(_) => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

// Blocks the calling thread — winit owns the event loop for the life of
// the program, so this is meant to be the last call in main().
pub fn run(config: WindowConfig) {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::new(config);
    event_loop
        .run_app(&mut app)
        .expect("event loop exited with an error");
}
