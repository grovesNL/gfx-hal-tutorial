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
    queue, Adapter, Backend, Capability, Gpu, Graphics, Instance, PhysicalDevice, QueueFamily, Surface, format, Backbuffer, SwapchainConfig, Device,
};
use winit::{dpi, ControlFlow, Event, EventsLoop, Window, WindowBuilder, WindowEvent};

static WINDOW_NAME: &str = "06_swap_chain_creation";

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
}

struct HelloTriangleApplication {
    _format: format::Format,
    _backbuffer: Backbuffer<back::Backend>,
    swapchain: Option<<back::Backend as Backend>::Swapchain>,
    _command_queues: Vec<queue::CommandQueue<back::Backend, Graphics>>,
    device: <back::Backend as Backend>::Device,
    _surface: <back::Backend as Backend>::Surface,
    _adapter: Adapter<back::Backend>,
    _instance: back::Instance,
    events_loop: EventsLoop,
    _window: Window,
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

impl HelloTriangleApplication {
    pub fn init() -> HelloTriangleApplication {
        let (window, events_loop) = HelloTriangleApplication::init_window();
        let (_instance, _adapter, _surface, device, _command_queues, swapchain, _backbuffer, _format) = HelloTriangleApplication::init_hal(&window);

        HelloTriangleApplication {
            _format,
            _backbuffer,
            swapchain: Some(swapchain),
            _command_queues,
            device,
            _surface,
            _adapter,
            _instance,
            events_loop,
            _window: window,
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

    fn init_hal(window: &Window) -> (back::Instance, Adapter<back::Backend>, <back::Backend as Backend>::Surface, <back::Backend as Backend>::Device, Vec<queue::CommandQueue<back::Backend, Graphics>>, <back::Backend as Backend>::Swapchain, Backbuffer<back::Backend>, format::Format) {
        let instance = HelloTriangleApplication::create_instance();
        let mut adapter = HelloTriangleApplication::pick_adapter(&instance);
        let mut surface = HelloTriangleApplication::create_surface(&instance, window);
        let (device, command_queues) = HelloTriangleApplication::create_device_with_graphics_queues(&mut adapter, &surface);
        let (swapchain, backbuffer, format) = HelloTriangleApplication::create_swap_chain(&adapter, &device, &mut surface, None);

        (instance, adapter, surface, device, command_queues, swapchain, backbuffer, format)
    }

    fn create_instance() -> back::Instance {
        back::Instance::create(WINDOW_NAME, 1)
    }

    fn find_queue_families(adapter: &Adapter<back::Backend>) -> QueueFamilyIds {
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
        HelloTriangleApplication::find_queue_families(adapter).is_complete()
    }

    fn pick_adapter(instance: &back::Instance) -> Adapter<back::Backend> {
        let adapters = instance.enumerate_adapters();
        for adapter in adapters {
            if HelloTriangleApplication::is_adapter_suitable(&adapter) {
                return adapter;
            }
        }
        panic!("No suitable adapter");
    }

    fn create_surface(
        instance: &back::Instance,
        window: &Window,
    ) -> <back::Backend as Backend>::Surface {
        instance.create_surface(window)
    }

    // we have an additional check to make: make sure the queue family selected supports presentation to the surface we have created
    fn create_device_with_graphics_queues(
        adapter: &mut Adapter<back::Backend>,
        surface: &<back::Backend as Backend>::Surface,
    ) -> (
        <back::Backend as Backend>::Device,
        Vec<queue::CommandQueue<back::Backend, Graphics>>,
    ) {
        let family = adapter
            .queue_families
            .iter()
            .find(|family| {
                Graphics::supported_by(family.queue_type())
                    && family.max_queues() > 0
                    && surface.supports_queue_family(family)
            }).expect("Could not find a queue family supporting graphics.");

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

    fn create_swap_chain(
        adapter: &Adapter<back::Backend>,
        device: &<back::Backend as Backend>::Device,
        surface: &mut <back::Backend as Backend>::Surface,
        previous_swapchain: Option<<back::Backend as Backend>::Swapchain>,
    ) -> (
        <back::Backend as Backend>::Swapchain,
        Backbuffer<back::Backend>,
        format::Format,
    ) {
        let (caps, formats, _present_modes) = surface.compatibility(&adapter.physical_device);

        let format = formats.map_or(format::Format::Rgba8Srgb, |formats| {
            formats
                .iter()
                .find(|format| format.base_format().1 == format::ChannelType::Srgb)
                .map(|format| *format)
                .unwrap_or(formats[0])
        });

        let swap_config = SwapchainConfig::from_caps(&caps, format);

        let (swapchain, backbuffer) = device.create_swapchain(surface, swap_config, previous_swapchain);

        (swapchain, backbuffer, format)
    }

    fn clean_up(&mut self) {
        //swapchain must be explicitly destroyed
        let swapchain = self.swapchain.take().unwrap();
        self.device.destroy_swapchain(swapchain);
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
