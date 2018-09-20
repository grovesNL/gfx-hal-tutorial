#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;
extern crate gfx_hal as hal;
extern crate winit;

use winit::{dpi, ControlFlow, Event, EventsLoop, Window, WindowBuilder, WindowEvent};

static WINDOW_NAME: &str = "00_base_code";

fn main() {
    let mut application = HelloTriangleApplication::init();
    application.run();
}

struct HelloTriangleApplication {
    // Rust drops struct fields in the order in which they are declared
    // we'll order fields in the order we would want them to be destroyed
    // for most fields, it doesn't matter, but places where it matters will be pointed out
    events_loop: EventsLoop,
    _window: Window,
}

impl HelloTriangleApplication {
    pub fn init() -> HelloTriangleApplication {
        let (_window, events_loop) = HelloTriangleApplication::init_window();
        HelloTriangleApplication::init_hal();

        HelloTriangleApplication {
            events_loop,
            _window,
        }
    }

    fn init_window() -> (Window, EventsLoop) {
        let events_loop = EventsLoop::new();
        let window_builder = WindowBuilder::new()
            .with_dimensions(dpi::LogicalSize::new(1024., 768.))
            .with_title(WINDOW_NAME.to_string());
        let window = window_builder.build(&events_loop).unwrap();
        (window, events_loop)
    }

    fn init_hal() {}

    fn clean_up(&self) {
        // winit handles window destruction
    }

    fn main_loop(&mut self) {
        self.events_loop.run_forever(|event| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => ControlFlow::Break,
            _ => ControlFlow::Continue,
        });
    }

    pub fn run(&mut self) {
        self.main_loop();
        self.clean_up();
    }
}

