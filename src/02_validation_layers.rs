extern crate env_logger;
#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;
extern crate gfx_hal as hal;
extern crate winit;

use winit::{dpi, ControlFlow, Event, EventsLoop, Window, WindowBuilder, WindowEvent};

static WINDOW_NAME: &str = "02_validation_layers";

fn main() {
    // if building in debug mode, vulkan backend initializes standard validation layers
    // all we need to do is enable logging
    // run the program like so to print all logs of level 'warn' and above:
    // bash: RUST_LOG=warn && cargo run --bin 02_validation_layers --features vulkan
    // powershell: $env:RUST_LOG="warn"; cargo run --bin 02_validation_layers --features vulkan
    // see: https://docs.rs/env_logger/0.5.13/env_logger/
    env_logger::init();
    let mut application = HelloTriangleApplication::init();
    application.run();
    application.clean_up();
}

struct WindowState {
    events_loop: EventsLoop,
    _window: Window,
}

struct HalState {
    _instance: back::Instance,
}

impl HalState {
    fn clean_up(self) {}
}

struct HelloTriangleApplication {
    hal_state: HalState,
    window_state: WindowState,
}

impl HelloTriangleApplication {
    pub fn init() -> HelloTriangleApplication {
        let window_state = HelloTriangleApplication::init_window();
        let hal_state = HelloTriangleApplication::init_hal();

        HelloTriangleApplication {
            hal_state,
            window_state,
        }
    }

    fn init_window() -> WindowState {
        let events_loop = EventsLoop::new();
        let window_builder = WindowBuilder::new()
            .with_dimensions(dpi::LogicalSize::new(1024., 768.))
            .with_title(WINDOW_NAME.to_string());
        let window = window_builder.build(&events_loop).unwrap();
        WindowState {
            events_loop,
            _window: window,
        }
    }

    fn init_hal() -> HalState {
        let instance = HelloTriangleApplication::create_instance();
        HalState {
            _instance: instance,
        }
    }

    fn create_instance() -> back::Instance {
        back::Instance::create(WINDOW_NAME, 1)
    }

    fn main_loop(&mut self) {
        self.window_state
            .events_loop
            .run_forever(|event| match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => ControlFlow::Break,
                _ => ControlFlow::Continue,
            });
    }

    fn run(&mut self) {
        self.main_loop();
    }

    fn clean_up(self) {
        self.hal_state.clean_up();
    }
}

