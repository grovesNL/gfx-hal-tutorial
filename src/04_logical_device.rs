extern crate env_logger;
#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;
extern crate gfx_hal as hal;
extern crate winit;

use hal::{
    queue, Adapter, Backend, Capability, Gpu, Graphics, Instance, PhysicalDevice, QueueFamily,
};
use winit::{dpi, ControlFlow, Event, EventsLoop, Window, WindowBuilder, WindowEvent};

static WINDOW_NAME: &str = "01_instance_creation";

fn main() {
    // if building in debug mode, vulkan backend initializes standard validation layers
    // all we need to do is enable logging
    // run the program like so to print all logs of level 'warn' and above:
    // bash: RUST_LOG=warn && cargo run --bin 02_validation_layers --features vulkan
    // powershell: $env:RUST_LOG="warn"; cargo run --bin 02_validation_layers --features vulkan
    // see: https://docs.rs/env_logger/0.5.13/env_logger/
    env_logger::init();
    let mut application = ApplicationState::init();
    application.run();
}

struct ApplicationState {
    _window: Window,
    events_loop: EventsLoop,
    _instance: back::Instance,
    _adapter: Adapter<back::Backend>,
    _device: <back::Backend as Backend>::Device,
    command_queues: Vec<queue::CommandQueue<back::Backend, Graphics>>,

}

#[derive(Default)]
struct QueueFamilyIds {
    graphics_family: Option<queue::QueueFamilyId>,
}

impl QueueFamilyIds {
    fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
    }
}

impl ApplicationState {
    pub fn init() -> ApplicationState {
        let (_window, events_loop) = ApplicationState::init_window();
        let (_instance, _adapter, _device, command_queues) = ApplicationState::init_hal();

        ApplicationState {
            _window,
            events_loop,
            _instance,
            _adapter,
            _device,
            command_queues,
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

    fn init_hal() -> (back::Instance, Adapter<back::Backend>, <back::Backend as Backend>::Device, Vec<queue::CommandQueue<back::Backend, Graphics>>) {
        let instance = ApplicationState::create_instance();
        let mut adapter = ApplicationState::pick_adapter(&instance);

        let (device, command_queues) = ApplicationState::create_device_with_graphics_queues(&mut adapter);

        (instance, adapter, device, command_queues)
    }

    fn create_instance() -> back::Instance {
        back::Instance::create(WINDOW_NAME, 1)
    }

    fn find_queue_families(adapter: &hal::Adapter<back::Backend>) -> QueueFamilyIds {
        let mut queue_family_ids = QueueFamilyIds::default();

        for queue_family in &adapter.queue_families {
            if queue_family.max_queues() > 0 && queue_family.supports_graphics() {
                queue_family_ids.graphics_family = Some(queue_family.id());
            }

            if queue_family_ids.is_complete() {
                break;
            }
        }

        queue_family_ids
    }

    fn is_adapter_suitable(adapter: &Adapter<back::Backend>) -> bool {
        ApplicationState::find_queue_families(adapter).is_complete()
    }

    fn pick_adapter(instance: &back::Instance) -> Adapter<back::Backend> {
        let adapters = instance.enumerate_adapters();
        for adapter in adapters {
            if ApplicationState::is_adapter_suitable(&adapter) {
                return adapter;
            }
        }
        panic!("No suitable adapter");
    }

    fn create_device_with_graphics_queues(
        adapter: &mut Adapter<back::Backend>,
    ) -> (
        <back::Backend as Backend>::Device,
        Vec<queue::CommandQueue<back::Backend, Graphics>>,
    ) {
        let family = adapter
            .queue_families
            .iter()
            .find(|family| Graphics::supported_by(family.queue_type()) && family.max_queues() > 0)
            .expect("Could not find a queue family supporting graphics.");

        // we only want to create a single queue
        let priorities = vec![1.0; 1];

        let families = [(family, priorities.as_slice())];

        let Gpu { device, mut queues } = adapter
            .physical_device
            .open(&families)
            .expect("Could not create device.");

        let mut queue_group = queues
            .take::<Graphics>(family.id())
            .expect("Could not take ownership of relevant queue group.");

        let command_queues: Vec<_> = queue_group.queues.drain(..1).collect();

        (device, command_queues)
    }

    fn clean_up(&self) {
        // command queues will drop automatically, but we need to give struct hint to make sure it drops before instance is dropped
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

