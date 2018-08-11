#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;
extern crate gfx_hal as hal;
extern crate winit;

static WINDOW_NAME: &str = "02_validation_layers";

fn main() {
    let (_window, events_loop) = init_window();
    let _instance = init_hal();
    main_loop(events_loop);
}

fn init_window() -> (winit::Window, winit::EventsLoop) {
    let events_loop = winit::EventsLoop::new();
    let window_builder = winit::WindowBuilder::new()
        .with_dimensions(winit::dpi::LogicalSize::new(1024., 768.))
        .with_title(WINDOW_NAME.to_string());
    let window = window_builder.build(&events_loop).unwrap();
    (window, events_loop)
}

fn init_hal() -> back::Instance {
    // if using Vulkan, HAL loads `VK_LAYER_LUNARG_standard_validation` during instance creation
    // user can specify additional layers by setting environment variables
    // some backends may not have Vulkan-esque validation layers!
    // end result: nothing changes in terms of code
    back::Instance::create(WINDOW_NAME, 1)
}

fn main_loop(mut events_loop: winit::EventsLoop) {
    events_loop.run_forever(|event| match event {
        winit::Event::WindowEvent {
            event: winit::WindowEvent::CloseRequested,
            ..
        } => winit::ControlFlow::Break,
        _ => winit::ControlFlow::Continue,
    });
}