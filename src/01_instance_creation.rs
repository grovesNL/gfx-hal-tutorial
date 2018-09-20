#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;
extern crate gfx_hal as hal;
extern crate winit;

use winit::{dpi, ControlFlow, Event, EventsLoop, Window, WindowBuilder, WindowEvent};

static WINDOW_NAME: &str = "01_instance_creation";

fn main() {
    let mut application = HelloTriangleApplication::init();
    application.run();
}

struct HelloTriangleApplication {
    _instance: back::Instance,
    events_loop: EventsLoop,
    _window: Window,
}

impl HelloTriangleApplication {
    pub fn init() -> HelloTriangleApplication {
        let (_window, events_loop) = HelloTriangleApplication::init_window();
        let _instance = HelloTriangleApplication::init_hal();

        HelloTriangleApplication {
            _instance,
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

    fn init_hal() -> back::Instance {
        let instance = HelloTriangleApplication::create_instance();

        instance
    }

    fn create_instance() -> back::Instance {
        back::Instance::create(WINDOW_NAME, 1)
    }

    fn clean_up(&self) {
        // instance will drop automatically
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

