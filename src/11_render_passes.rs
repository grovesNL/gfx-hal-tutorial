extern crate env_logger;
#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;
extern crate gfx_hal as hal;
extern crate glsl_to_spirv;
extern crate winit;

use hal::{
    format, image, pass, pso, queue, window, Adapter, Backbuffer, Backend, Capability, Device, Gpu,
    Graphics, Instance, PhysicalDevice, Primitive, QueueFamily, Surface, SwapchainConfig,
};
use std::io::Read;
use winit::{dpi, ControlFlow, Event, EventsLoop, Window, WindowBuilder, WindowEvent};

static WINDOW_NAME: &str = "11_render_passes";

fn main() {
    env_logger::init();
    let mut application = HelloTriangleApplication::init();
    application.run();
    application.clean_up();
}

struct WindowState {
    events_loop: EventsLoop,
    window: Window,
}

struct HalState {
    descriptor_set_layouts: Vec<<back::Backend as Backend>::DescriptorSetLayout>,
    pipeline_layout: <back::Backend as Backend>::PipelineLayout,
    render_pass: <back::Backend as Backend>::RenderPass,
    frame_images: Vec<(
        <back::Backend as Backend>::Image,
        <back::Backend as Backend>::ImageView,
    )>,
    _format: format::Format,
    swapchain: <back::Backend as Backend>::Swapchain,
    _command_queues: Vec<queue::CommandQueue<back::Backend, Graphics>>,
    device: <back::Backend as Backend>::Device,
    _surface: <back::Backend as Backend>::Surface,
    _adapter: Adapter<back::Backend>,
    _instance: back::Instance,
}

impl HalState {
    fn clean_up(self) {
        let device = &self.device;

        for descriptor_set_layout in self.descriptor_set_layouts {
            device.destroy_descriptor_set_layout(descriptor_set_layout);
        }

        device.destroy_pipeline_layout(self.pipeline_layout);

        device.destroy_render_pass(self.render_pass);

        for (_, image_view) in self.frame_images.into_iter() {
            device.destroy_image_view(image_view);
        }

        device.destroy_swapchain(self.swapchain);
    }
}

struct HelloTriangleApplication {
    hal_state: HalState,
    window_state: WindowState,
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
        let window_state = HelloTriangleApplication::init_window();
        let hal_state = HelloTriangleApplication::init_hal(&window_state.window);

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
            window,
        }
    }

    fn init_hal(window: &Window) -> HalState {
        let instance = HelloTriangleApplication::create_instance();
        let mut adapter = HelloTriangleApplication::pick_adapter(&instance);
        let mut surface = HelloTriangleApplication::create_surface(&instance, window);
        let (device, command_queues) =
            HelloTriangleApplication::create_device_with_graphics_queues(&mut adapter, &surface);
        let (swapchain, extent, backbuffer, format) =
            HelloTriangleApplication::create_swap_chain(&adapter, &device, &mut surface, None);
        let frame_images =
            HelloTriangleApplication::create_image_views(backbuffer, format, &device);
        let render_pass = HelloTriangleApplication::create_render_pass(&device, Some(format));
        let (descriptor_set_layouts, pipeline_layout) =
            HelloTriangleApplication::create_graphics_pipeline(&device, extent);

        HalState {
            descriptor_set_layouts,
            pipeline_layout,
            render_pass,
            frame_images,
            _format: format,
            swapchain,
            _command_queues: command_queues,
            device,
            _surface: surface,
            _adapter: adapter,
            _instance: instance,
        }
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
        window::Extent2D,
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
        let extent = swap_config.extent;
        let (swapchain, backbuffer) =
            device.create_swapchain(surface, swap_config, previous_swapchain);

        (swapchain, extent, backbuffer, format)
    }

    fn create_image_views(
        backbuffer: Backbuffer<back::Backend>,
        format: format::Format,
        device: &<back::Backend as Backend>::Device,
    ) -> Vec<(
        <back::Backend as Backend>::Image,
        <back::Backend as Backend>::ImageView,
    )> {
        match backbuffer {
            window::Backbuffer::Images(images) => images
                .into_iter()
                .map(|image| {
                    let image_view = match device.create_image_view(
                        &image,
                        image::ViewKind::D2,
                        format,
                        format::Swizzle::NO,
                        image::SubresourceRange {
                            aspects: format::Aspects::COLOR,
                            levels: 0..1,
                            layers: 0..1,
                        },
                    ) {
                        Ok(image_view) => image_view,
                        Err(_) => panic!("Error creating image view for an image!"),
                    };

                    (image, image_view)
                }).collect(),
            _ => unimplemented!(),
        }
    }

    fn create_render_pass(
        device: &<back::Backend as Backend>::Device,
        format: Option<format::Format>,
    ) -> <back::Backend as Backend>::RenderPass {
        let samples: u8 = 1;

        let ops = pass::AttachmentOps {
            load: pass::AttachmentLoadOp::Clear,
            store: pass::AttachmentStoreOp::Store,
        };

        let stencil_ops = pass::AttachmentOps::DONT_CARE;

        let layouts = image::Layout::Undefined..image::Layout::Present;

        let color_attachment = pass::Attachment {
            format,
            samples,
            ops,
            stencil_ops,
            layouts,
        };

        let color_attachment_ref: pass::AttachmentRef = (0, image::Layout::ColorAttachmentOptimal);

        // hal assumes pipeline bind point is GRAPHICS
        let subpass = pass::SubpassDesc {
            colors: &[color_attachment_ref],
            depth_stencil: None,
            inputs: &[],
            resolves: &[],
            preserves: &[],
        };

        device.create_render_pass(&[color_attachment], &[subpass], &[])
    }

    fn create_graphics_pipeline(
        device: &<back::Backend as Backend>::Device,
        extent: window::Extent2D,
    ) -> (
        Vec<<back::Backend as Backend>::DescriptorSetLayout>,
        <back::Backend as Backend>::PipelineLayout,
    ) {
        let vert_shader_code = glsl_to_spirv::compile(
            include_str!("09_shader_base.vert"),
            glsl_to_spirv::ShaderType::Vertex,
        ).expect("Error compiling vertex shader code.")
        .bytes()
        .map(|b| b.unwrap())
        .collect::<Vec<u8>>();

        let frag_shader_code = glsl_to_spirv::compile(
            include_str!("09_shader_base.frag"),
            glsl_to_spirv::ShaderType::Fragment,
        ).expect("Error compiling fragment shader code.")
        .bytes()
        .map(|b| b.unwrap())
        .collect::<Vec<u8>>();

        let vert_shader_module = device
            .create_shader_module(&vert_shader_code)
            .expect("Error creating shader module.");
        let frag_shader_module = device
            .create_shader_module(&frag_shader_code)
            .expect("Error creating fragment module.");

        let (ds_layouts, pipeline_layout) = {
            let (vs_entry, fs_entry) = (
                pso::EntryPoint::<back::Backend> {
                    entry: "main",
                    module: &vert_shader_module,
                    specialization: &[],
                },
                pso::EntryPoint::<back::Backend> {
                    entry: "main",
                    module: &frag_shader_module,
                    specialization: &[],
                },
            );

            let _shaders = pso::GraphicsShaderSet {
                vertex: vs_entry,
                hull: None,
                domain: None,
                geometry: None,
                fragment: Some(fs_entry),
            };

            let _rasterizer = pso::Rasterizer {
                depth_clamping: false,
                polygon_mode: pso::PolygonMode::Fill,
                cull_face: <pso::Face>::BACK,
                front_face: pso::FrontFace::Clockwise,
                depth_bias: None,
                conservative: false,
            };

            let _vertex_buffers: Vec<pso::VertexBufferDesc> = Vec::new();
            let _attributes: Vec<pso::AttributeDesc> = Vec::new();

            let _input_assembler = pso::InputAssemblerDesc::new(Primitive::TriangleList);

            let _blender = {
                let blend_state = pso::BlendState::On {
                    color: pso::BlendOp::Add {
                        src: pso::Factor::One,
                        dst: pso::Factor::Zero,
                    },
                    alpha: pso::BlendOp::Add {
                        src: pso::Factor::One,
                        dst: pso::Factor::Zero,
                    },
                };

                pso::BlendDesc {
                    logic_op: Some(pso::LogicOp::Copy),
                    targets: vec![pso::ColorBlendDesc(pso::ColorMask::ALL, blend_state)],
                }
            };

            let _depth_stencil = pso::DepthStencilDesc {
                depth: pso::DepthTest::Off,
                depth_bounds: false,
                stencil: pso::StencilTest::Off,
            };

            let _multisampling: Option<pso::Multisampling> = None;

            let _baked_states = pso::BakedStates {
                viewport: Some(pso::Viewport {
                    rect: pso::Rect {
                        x: 0,
                        y: 0,
                        w: extent.width as i16,
                        h: extent.width as i16,
                    },
                    depth: (0.0..1.0),
                }),
                scissor: Some(pso::Rect {
                    x: 0,
                    y: 0,
                    w: extent.width as i16,
                    h: extent.height as i16,
                }),
                blend_color: None,
                depth_bounds: None,
            };

            let bindings = Vec::<pso::DescriptorSetLayoutBinding>::new();
            let immutable_samplers = Vec::<<back::Backend as Backend>::Sampler>::new();
            let ds_layouts: Vec<<back::Backend as Backend>::DescriptorSetLayout> =
                vec![device.create_descriptor_set_layout(bindings, immutable_samplers)];
            let push_constants = Vec::<(pso::ShaderStageFlags, std::ops::Range<u32>)>::new();
            let layout = device.create_pipeline_layout(&ds_layouts, push_constants);

            // our goal is to fill out this entire struct
            //    let desc = pso::GraphicsPipelineDesc {
            //        shaders,
            //        rasterizer,
            //        vertex_buffers,
            //        attributes,
            //        input_assembler,
            //        blender,
            //        depth_stencil,
            //        multisampling,
            //        baked_states,
            //        layout,
            //        subpass,
            //        flags,
            //        parent,
            //    };

            (ds_layouts, layout)
        };

        device.destroy_shader_module(vert_shader_module);
        device.destroy_shader_module(frag_shader_module);

        (ds_layouts, pipeline_layout)
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

