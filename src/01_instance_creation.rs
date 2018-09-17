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
    let (window, events_loop) = init_window();
    let mut application_state = ApplicationState::new(window, events_loop);
    application_state.main_loop();
    application_state.clean_up();
}

fn init_window() -> (Window, EventsLoop) {
    let events_loop = EventsLoop::new();
    let window_builder = WindowBuilder::new()
        .with_dimensions(dpi::LogicalSize::new(1024., 768.))
        .with_title(WINDOW_NAME.to_string());
    let window = window_builder.build(&events_loop).unwrap();
    (window, events_loop)
}

struct ApplicationState {
    _window: Window,
    events_loop: EventsLoop,
    _instance: back::Instance,
}

impl ApplicationState {
    pub fn new(_window: Window, events_loop: EventsLoop) -> ApplicationState {
        let _instance = ApplicationState::init_hal();

        ApplicationState {
            _window,
            events_loop,
            _instance,
        }
    }

    fn create_instance() -> back::Instance {
        back::Instance::create(WINDOW_NAME, 1)
    }

    fn init_hal() -> back::Instance {
        let _instance = ApplicationState::create_instance();

        _instance
    }

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
}
