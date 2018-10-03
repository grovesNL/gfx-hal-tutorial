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
    queue, Adapter, Backend, Capability, Gpu, Graphics, Instance, PhysicalDevice, QueueFamily, Surface, format, Backbuffer, SwapchainConfig, Device,
};
use winit::{dpi, ControlFlow, Event, EventsLoop, Window, WindowBuilder, WindowEvent};
use std::io::Read;

static WINDOW_NAME: &str = "09_shader_modules";

fn main() {
    env_logger::init();
    let mut application = HelloTriangleApplication::init();
    application.run();
}

struct HelloTriangleApplication {
    submission_command_buffers: Option<Vec<hal::command::Submit<back::Backend, hal::Graphics, hal::command::MultiShot, hal::command::Primary,>>>,
    command_pool: Option<hal::pool::CommandPool<back::Backend, hal::Graphics>>,
    swapchain_framebuffers: Option<Vec<<back::Backend as hal::Backend>::Framebuffer>>,
    gfx_pipeline: Option<<back::Backend as hal::Backend>::GraphicsPipeline>,
    descriptor_set_layouts: Option<Vec<<back::Backend as hal::Backend>::DescriptorSetLayout>>,
    pipeline_layout: Option<<back::Backend as hal::Backend>::PipelineLayout>,
    render_pass: Option<<back::Backend as hal::Backend>::RenderPass>,
    frame_images: Option<Vec<(<back::Backend as Backend>::Image, <back::Backend as Backend>::ImageView,)>>,
    _format: format::Format,
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
        let (_instance, _adapter, _surface, device, _command_queues, swapchain, _format, frame_images, render_pass, pipeline_layout, descriptor_set_layout, gfx_pipeline, swapchain_framebuffers, command_pool, submission_command_buffers) = HelloTriangleApplication::init_hal(&window);

        HelloTriangleApplication {
            submission_command_buffers: Some(submission_command_buffers)
            command_pool: Some(command_pool),
            swapchain_framebuffers: Some(swapchain_framebuffers),
            gfx_pipeline: Some(gfx_pipeline),
            descriptor_set_layouts: Some(descriptor_set_layout),
            pipeline_layout: Some(pipeline_layout),
            render_pass: Some(render_pass),
            frame_images: Some(frame_images),
            _format,
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

    fn init_hal(window: &Window) -> (
        back::Instance,
        Adapter<back::Backend>,
        <back::Backend as Backend>::Surface,
        <back::Backend as Backend>::Device,
        Vec<queue::CommandQueue<back::Backend, Graphics>>,
        <back::Backend as Backend>::Swapchain,
        format::Format,
        Vec<(<back::Backend as Backend>::Image, <back::Backend as Backend>::ImageView,)>,
        <back::Backend as Backend>::RenderPass,
        <back::Backend as hal::Backend>::PipelineLayout,
        Vec<<back::Backend as hal::Backend>::DescriptorSetLayout>,
        <back::Backend as hal::Backend>::GraphicsPipeline,
        Vec<<back::Backend as hal::Backend>::Framebuffer>,
        hal::pool::CommandPool<back::Backend, hal::Graphics>,
        Vec<hal::command::Submit<back::Backend, hal::Graphics, hal::command::MultiShot, hal::command::Primary,>,
    ) {
        let instance = HelloTriangleApplication::create_instance();
        let mut adapter = HelloTriangleApplication::pick_adapter(&instance);
        let mut surface = HelloTriangleApplication::create_surface(&instance, window);
        let (device, command_queues) = HelloTriangleApplication::create_device_with_graphics_queues(&mut adapter, &surface);
        let (swapchain, extent, backbuffer, format) = HelloTriangleApplication::create_swap_chain(&adapter, &device, &mut surface, None);
        let frame_images = HelloTriangleApplication::create_image_views(backbuffer, format, &device);
        let render_pass = HelloTriangleApplication::create_render_pass(&device, Some(format));
        let (ds_layouts, pipeline_layout, gfx_pipeline) = HelloTriangleApplication::create_graphics_pipeline(&device, extent, &render_pass);
        let swapchain_framebuffers = HelloTriangleApplication::create_framebuffers(&device, &render_pass, &frame_images, extent);
        let mut command_pool = HelloTriangleApplication::create_command_pool(&device, queue_type, qf_id);
        let submission_command_buffers = HelloTriangleApplication::create_command_buffers(
            &mut command_pool,
            &render_pass,
            &swapchain_framebuffers,
            extent,
            &gfx_pipeline,
        );

        (instance,
         adapter,
         surface,
         device,
         command_queues,
         swapchain,
         format,
         frame_images,
         render_pass,
         pipeline_layout,
         ds_layouts,
         gfx_pipeline,
         swapchain_framebuffers,
         command_pool,
         submission_command_buffers
        )
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
        adapter: &hal::Adapter<back::Backend>,
        device: &<back::Backend as hal::Backend>::Device,
        surface: &mut <back::Backend as hal::Backend>::Surface,
        previous_swapchain: Option<<back::Backend as hal::Backend>::Swapchain>,
    ) -> (
        <back::Backend as hal::Backend>::Swapchain,
        hal::window::Extent2D,
        hal::Backbuffer<back::Backend>,
        hal::format::Format,
    ) {
        let (caps, formats, _present_modes) = surface.compatibility(&adapter.physical_device);

        let format = formats.map_or(hal::format::Format::Rgba8Srgb, |formats| {
            formats
                .iter()
                .find(|format| format.base_format().1 == hal::format::ChannelType::Srgb)
                .map(|format| *format)
                .unwrap_or(formats[0])
        });

        let swap_config = SwapchainConfig::from_caps(&caps, format);
        let extent = swap_config.extent.clone();
        let (swapchain, backbuffer) = device.create_swapchain(surface, swap_config, previous_swapchain);

        (swapchain, extent, backbuffer, format)
    }


    fn create_image_views(backbuffer: Backbuffer<back::Backend>, format: format::Format, device: &<back::Backend as Backend>::Device) -> Vec<(
        <back::Backend as hal::Backend>::Image,
        <back::Backend as hal::Backend>::ImageView,
    )> {
        match backbuffer {
            hal::window::Backbuffer::Images(images) => images
                .into_iter()
                .map(|image| {
                    let image_view = match device.create_image_view(
                        &image,
                        hal::image::ViewKind::D2,
                        format,
                        hal::format::Swizzle::NO,
                        hal::image::SubresourceRange {
                            aspects: hal::format::Aspects::COLOR,
                            levels: 0..1,
                            layers: 0..1,
                        },
                    ) {
                        Ok(image_view) => image_view,
                        Err(_) => panic!("Error creating image view for an image!"),
                    };

                    (image, image_view)
                }).collect(),
            // OpenGL case, where backbuffer is a framebuffer, not implemented currently
            _ => unimplemented!(),
        }
    }

    fn create_render_pass(
        device: &<back::Backend as hal::Backend>::Device,
        format: Option<hal::format::Format>,
    ) -> <back::Backend as hal::Backend>::RenderPass {
        let samples: u8 = 1;

        let ops = hal::pass::AttachmentOps {
            load: hal::pass::AttachmentLoadOp::Clear,
            store: hal::pass::AttachmentStoreOp::Store,
        };

        let stencil_ops = hal::pass::AttachmentOps::DONT_CARE;

        let layouts = hal::image::Layout::Undefined..hal::image::Layout::Present;

        let color_attachment = hal::pass::Attachment {
            format,
            samples,
            ops,
            stencil_ops,
            layouts,
        };

        let color_attachment_ref: hal::pass::AttachmentRef =
            (0, hal::image::Layout::ColorAttachmentOptimal);

        // hal assumes pipeline bind point is GRAPHICS
        let subpass = hal::pass::SubpassDesc {
            colors: &[color_attachment_ref],
            depth_stencil: None,
            inputs: &[],
            resolves: &[],
            preserves: &[],
        };

        device.create_render_pass(&[color_attachment], &[subpass], &[])
    }

    fn create_graphics_pipeline(
        device: &<back::Backend as hal::Backend>::Device,
        extent: hal::window::Extent2D,
        render_pass: &<back::Backend as hal::Backend>::RenderPass,
    ) -> (
        Vec<<back::Backend as hal::Backend>::DescriptorSetLayout>,
        <back::Backend as hal::Backend>::PipelineLayout,
        <back::Backend as hal::Backend>::GraphicsPipeline,
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

        let (ds_layouts, pipeline_layout, gfx_pipeline) = {
            let (vs_entry, fs_entry) = (
                hal::pso::EntryPoint::<back::Backend> {
                    entry: "main",
                    module: &vert_shader_module,
                    specialization: &[],
                },
                hal::pso::EntryPoint::<back::Backend> {
                    entry: "main",
                    module: &frag_shader_module,
                    specialization: &[],
                },
            );

            let shaders = hal::pso::GraphicsShaderSet {
                vertex: vs_entry,
                hull: None,
                domain: None,
                geometry: None,
                fragment: Some(fs_entry),
            };

            let rasterizer = hal::pso::Rasterizer {
                depth_clamping: false,
                polygon_mode: hal::pso::PolygonMode::Fill,
                cull_face: <hal::pso::Face>::BACK,
                front_face: hal::pso::FrontFace::Clockwise,
                depth_bias: None,
                conservative: false,
            };

            let vertex_buffers: Vec<hal::pso::VertexBufferDesc> = Vec::new();
            let attributes: Vec<hal::pso::AttributeDesc> = Vec::new();

            let input_assembler = hal::pso::InputAssemblerDesc::new(hal::Primitive::TriangleList);

            let blender = {
                let blend_state = hal::pso::BlendState::On {
                    color: hal::pso::BlendOp::Add {
                        src: hal::pso::Factor::One,
                        dst: hal::pso::Factor::Zero,
                    },
                    alpha: hal::pso::BlendOp::Add {
                        src: hal::pso::Factor::One,
                        dst: hal::pso::Factor::Zero,
                    },
                };

                hal::pso::BlendDesc {
                    logic_op: Some(hal::pso::LogicOp::Copy),
                    targets: vec![hal::pso::ColorBlendDesc(
                        hal::pso::ColorMask::ALL,
                        blend_state,
                    )],
                }
            };

            let depth_stencil = hal::pso::DepthStencilDesc {
                depth: hal::pso::DepthTest::Off,
                depth_bounds: false,
                stencil: hal::pso::StencilTest::Off,
            };

            let multisampling: Option<hal::pso::Multisampling> = None;

            let baked_states = hal::pso::BakedStates {
                viewport: Some(hal::pso::Viewport {
                    rect: hal::pso::Rect {
                        x: 0,
                        y: 0,
                        w: extent.width as i16,
                        h: extent.width as i16,
                    },
                    depth: (0.0..1.0),
                }),
                scissor: Some(hal::pso::Rect {
                    x: 0,
                    y: 0,
                    w: extent.width as i16,
                    h: extent.height as i16,
                }),
                blend_color: None,
                depth_bounds: None,
            };

            let bindings = Vec::<hal::pso::DescriptorSetLayoutBinding>::new();
            let immutable_samplers = Vec::<<back::Backend as hal::Backend>::Sampler>::new();
            let ds_layouts: Vec<<back::Backend as hal::Backend>::DescriptorSetLayout> =
                vec![device.create_descriptor_set_layout(bindings, immutable_samplers)];
            let push_constants = Vec::<(hal::pso::ShaderStageFlags, std::ops::Range<u32>)>::new();
            let layout = device.create_pipeline_layout(&ds_layouts, push_constants);

            let subpass = hal::pass::Subpass {
                index: 0,
                main_pass: render_pass,
            };

            let flags = hal::pso::PipelineCreationFlags::empty();

            let parent = hal::pso::BasePipeline::None;

            let gfx_pipeline = {
                let desc = hal::pso::GraphicsPipelineDesc {
                    shaders,
                    rasterizer,
                    vertex_buffers,
                    attributes,
                    input_assembler,
                    blender,
                    depth_stencil,
                    multisampling,
                    baked_states,
                    layout: &layout,
                    subpass,
                    flags,
                    parent,
                };

                device
                    .create_graphics_pipeline(&desc, None)
                    .expect("failed to create graphics pipeline!")
            };

            (ds_layouts, layout, gfx_pipeline)
        };

        device.destroy_shader_module(vert_shader_module);
        device.destroy_shader_module(frag_shader_module);

        (ds_layouts, pipeline_layout, gfx_pipeline)
    }

    fn create_framebuffers(
        device: &<back::Backend as hal::Backend>::Device,
        render_pass: &<back::Backend as hal::Backend>::RenderPass,
        frame_images: &Vec<(
            <back::Backend as hal::Backend>::Image,
            <back::Backend as hal::Backend>::ImageView,
        )>,
        extent: hal::window::Extent2D,
    ) -> Vec<<back::Backend as hal::Backend>::Framebuffer> {
        let mut swapchain_framebuffers: Vec<<back::Backend as hal::Backend>::Framebuffer> = Vec::new();

        for (_, image_view) in frame_images.iter() {
            swapchain_framebuffers.push(
                device
                    .create_framebuffer(
                        render_pass,
                        vec![image_view],
                        hal::image::Extent {
                            width: extent.width as _,
                            height: extent.height as _,
                            depth: 1,
                        },
                    ).expect("failed to create framebuffer!"),
            );
        }

        swapchain_framebuffers
    }

    fn create_command_buffers<'a>(
        command_pool: &'a mut hal::pool::CommandPool<back::Backend, hal::Graphics>,
        render_pass: &<back::Backend as hal::Backend>::RenderPass,
        framebuffers: &Vec<<back::Backend as hal::Backend>::Framebuffer>,
        extent: hal::window::Extent2D,
        pipeline: &<back::Backend as hal::Backend>::GraphicsPipeline,
    ) -> Vec<
        hal::command::Submit<
            back::Backend,
            hal::Graphics,
            hal::command::MultiShot,
            hal::command::Primary,
        >,
    > {
        // reserve (allocate memory for) num number of primary command buffers
        command_pool.reserve(framebuffers.iter().count());

        let mut submission_command_buffers: Vec<
            hal::command::Submit<
                back::Backend,
                hal::Graphics,
                hal::command::MultiShot,
                hal::command::Primary,
            >,
        > = Vec::new();

        for fb in framebuffers.iter() {
            // command buffer will be returned in 'recording' state
            // Shot: how many times a command buffer can be submitted; we want MultiShot (allow submission multiple times)
            // Level: command buffer type (primary or secondary)
            // allow_pending_resubmit is set to true, as we want to allow simultaneous use per vulkan-tutorial
            let mut command_buffer: hal::command::CommandBuffer<
                back::Backend,
                hal::Graphics,
                hal::command::MultiShot,
                hal::command::Primary,
            > = command_pool.acquire_command_buffer(true);

            command_buffer.bind_graphics_pipeline(pipeline);
            {
                // begin render pass
                let render_area = hal::pso::Rect {
                    x: 0,
                    y: 0,
                    w: extent.width as _,
                    h: extent.height as _,
                };
                let clear_values = vec![hal::command::ClearValue::Color(
                    hal::command::ClearColor::Float([0.0, 0.0, 0.0, 4.0]),
                )];

                let mut render_pass_inline_encoder = command_buffer.begin_render_pass_inline(
                    render_pass,
                    fb,
                    render_area,
                    clear_values.iter(),
                );
                // HAL encoder draw command is best understood by seeing how it expands out:
                // vertex_count = vertices.end - vertices.start
                // instance_count = instances.end - instances.start
                // first_vertex = vertices.start
                // first_instance = instances.start
                render_pass_inline_encoder.draw(0..3, 0..1);
            }

            let submission_command_buffer = command_buffer.finish();
            submission_command_buffers.push(submission_command_buffer);
        }

        submission_command_buffers
    }

    fn create_command_pool(
        device: &<back::Backend as hal::Backend>::Device,
        queue_type: hal::queue::QueueType,
        qf_id: hal::queue::family::QueueFamilyId,
    ) -> hal::pool::CommandPool<back::Backend, hal::Graphics> {
        // raw command pool: a thin wrapper around command pools
        // strongly typed command pool: a safe wrapper around command pools, which ensures that only one command buffer is recorded at the same time from the current queue
        let raw_command_pool =
            device.create_command_pool(qf_id, hal::pool::CommandPoolCreateFlags::empty());

        // safety check necessary before creating a strongly typed command pool
        assert_eq!(hal::Graphics::supported_by(queue_type), true);
        unsafe { hal::pool::CommandPool::new(raw_command_pool) }
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
    }
}

fn take<T>(some_option: &mut Option<T>) -> T {
    some_option.take().unwrap()
}

impl Drop for HelloTriangleApplication {
    fn drop(&mut self) {
        let command_pool = take(&mut self.command_pool);
        self.device.destroy_command_pool(command_pool.into_raw());

        let swapchain_framebuffers = take(&mut self.swapchain_framebuffers);
        for fb in swapchain_framebuffers.into_iter() {
            self.device.destroy_framebuffer(fb);
        }

        let gfx_pipeline = take(&mut self.gfx_pipeline);
        self.device.destroy_graphics_pipeline(gfx_pipeline);

        let descriptor_set_layouts = take(&mut self.descriptor_set_layouts);
        for dsl in descriptor_set_layouts.into_iter() {
            self.device.destroy_descriptor_set_layout(dsl);
        }

        let pipeline_layout = take(&mut self.pipeline_layout);
        self.device.destroy_pipeline_layout(pipeline_layout);

        let render_pass = take(&mut self.render_pass);
        self.device.destroy_render_pass(render_pass);

        let frame_images = take(&mut self.frame_images);
        for (_, image_view) in frame_images.into_iter() {
            self.device.destroy_image_view(image_view);
        }

        let swapchain = take(&mut self.swapchain);
        self.device.destroy_swapchain(swapchain);
    }
}

