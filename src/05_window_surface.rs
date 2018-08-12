extern crate env_logger;
#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;
extern crate gfx_hal as hal;
extern crate winit;

use hal::{Capability, Instance, PhysicalDevice, QueueFamily, Surface};

static WINDOW_NAME: &str = "05_window_surface";

fn main() {
    env_logger::init();
    let (window, events_loop) = init_window();
    init_hal(&window);
    main_loop(events_loop);
    clean_up();
}

fn create_surface(
    instance: &back::Instance,
    window: &winit::Window,
) -> <back::Backend as hal::Backend>::Surface {
    instance.create_surface(window)
}

// we have an additional check to make: make sure the queue family selected supports presentation to the surface we have created
fn create_device_with_graphics_queues(
    adapter: &mut hal::Adapter<back::Backend>,
    surface: &<back::Backend as hal::Backend>::Surface,
) -> (
    <back::Backend as hal::Backend>::Device,
    Vec<hal::queue::CommandQueue<back::Backend, hal::Graphics>>,
) {
    let family = adapter
        .queue_families
        .iter()
        .find(|family| {
            hal::Graphics::supported_by(family.queue_type())
                && family.max_queues() > 0
                && surface.supports_queue_family(family)
        }).expect("Could not find a queue family supporting graphics.");

    // we only want to create a single queue
    let priorities = vec![1.0; 1];

    let families = [(family, priorities.as_slice())];

    let hal::Gpu { device, mut queues } = adapter
        .physical_device
        .open(&families)
        .expect("Could not create device.");

    let mut queue_group = queues
        .take::<hal::Graphics>(family.id())
        .expect("Could not take ownership of relevant queue group.");

    let command_queues: Vec<_> = queue_group.queues.drain(..1).collect();

    (device, command_queues)
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

fn is_adapter_suitable(adapter: &hal::Adapter<back::Backend>) -> bool {
    find_queue_families(adapter).is_complete()
}

fn pick_adapter(instance: &back::Instance) -> hal::Adapter<back::Backend> {
    let adapters = instance.enumerate_adapters();
    for adapter in adapters {
        if is_adapter_suitable(&adapter) {
            return adapter;
        }
    }
    panic!("No suitable adapter");
}

#[derive(Default)]
struct QueueFamilyIds {
    graphics_family: Option<hal::queue::QueueFamilyId>,
}

impl QueueFamilyIds {
    fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
    }
}

fn init_window() -> (winit::Window, winit::EventsLoop) {
    let events_loop = winit::EventsLoop::new();
    let window_builder = winit::WindowBuilder::new()
        .with_dimensions(winit::dpi::LogicalSize::new(1024., 768.))
        .with_title(WINDOW_NAME.to_string());
    let window = window_builder.build(&events_loop).unwrap();
    (window, events_loop)
}

fn create_instance() -> back::Instance {
    back::Instance::create(WINDOW_NAME, 1)
}

fn init_hal(window: &winit::Window) {
    let instance = create_instance();
    let surface = create_surface(&instance, window);
    let mut adapter = pick_adapter(&instance);
    let (_device, _command_queues) = create_device_with_graphics_queues(&mut adapter, &surface);
}

fn clean_up() {
    // HAL has implemented automatic destruction of the surface
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
