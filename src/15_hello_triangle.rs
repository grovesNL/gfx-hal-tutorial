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
    command, format, image, pass, pool, pso, queue, window, Adapter, Backbuffer, Backend,
    Capability, Device, Gpu, Graphics, Instance, PhysicalDevice, Primitive, QueueFamily, Surface,
    Swapchain, SwapchainConfig,
};
use std::io::Read;
use winit::{dpi, ControlFlow, Event, EventsLoop, Window, WindowBuilder, WindowEvent};

static WINDOW_NAME: &str = "09_shader_modules";
const MAX_FRAMES_IN_FLIGHT: usize = 2;

fn main() {
    env_logger::init();
    let mut application = HelloTriangleApplication::init();
    application.run();
}

struct HelloTriangleApplication {
    in_flight_fences: Option<Vec<<back::Backend as Backend>::Fence>>,
    render_finished_semaphores: Option<Vec<<back::Backend as Backend>::Semaphore>>,
    image_available_semaphores: Option<Vec<<back::Backend as Backend>::Semaphore>>,
    submission_command_buffers:
        Option<Vec<command::Submit<back::Backend, Graphics, command::MultiShot, command::Primary>>>,
    command_pool: Option<pool::CommandPool<back::Backend, Graphics>>,
    swapchain_framebuffers: Option<Vec<<back::Backend as Backend>::Framebuffer>>,
    gfx_pipeline: Option<<back::Backend as Backend>::GraphicsPipeline>,
    descriptor_set_layouts: Option<Vec<<back::Backend as Backend>::DescriptorSetLayout>>,
    pipeline_layout: Option<<back::Backend as Backend>::PipelineLayout>,
    render_pass: Option<<back::Backend as Backend>::RenderPass>,
    frame_images: Option<
        Vec<(
            <back::Backend as Backend>::Image,
            <back::Backend as Backend>::ImageView,
        )>,
    >,
    _format: format::Format,
    swapchain: Option<<back::Backend as Backend>::Swapchain>,
    command_queues: Vec<queue::CommandQueue<back::Backend, Graphics>>,
    device: <back::Backend as Backend>::Device,
    _surface: <back::Backend as Backend>::Surface,
    _adapter: Adapter<back::Backend>,
    _instance: back::Instance,
    events_loop: Option<EventsLoop>,
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
        let (
            _instance,
            _adapter,
            _surface,
            device,
            command_queues,
            swapchain,
            _format,
            frame_images,
            render_pass,
            pipeline_layout,
            descriptor_set_layout,
            gfx_pipeline,
            swapchain_framebuffers,
            command_pool,
            submission_command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
        ) = HelloTriangleApplication::init_hal(&window);

        HelloTriangleApplication {
            in_flight_fences: Some(in_flight_fences),
            render_finished_semaphores: Some(render_finished_semaphores),
            image_available_semaphores: Some(image_available_semaphores),
            submission_command_buffers: Some(submission_command_buffers),
            command_pool: Some(command_pool),
            swapchain_framebuffers: Some(swapchain_framebuffers),
            gfx_pipeline: Some(gfx_pipeline),
            descriptor_set_layouts: Some(descriptor_set_layout),
            pipeline_layout: Some(pipeline_layout),
            render_pass: Some(render_pass),
            frame_images: Some(frame_images),
            _format,
            swapchain: Some(swapchain),
            command_queues,
            device,
            _surface,
            _adapter,
            _instance,
            events_loop: Some(events_loop),
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

    fn init_hal(
        window: &Window,
    ) -> (
        back::Instance,
        Adapter<back::Backend>,
        <back::Backend as Backend>::Surface,
        <back::Backend as Backend>::Device,
        Vec<queue::CommandQueue<back::Backend, Graphics>>,
        <back::Backend as Backend>::Swapchain,
        format::Format,
        Vec<(
            <back::Backend as Backend>::Image,
            <back::Backend as Backend>::ImageView,
        )>,
        <back::Backend as Backend>::RenderPass,
        <back::Backend as Backend>::PipelineLayout,
        Vec<<back::Backend as Backend>::DescriptorSetLayout>,
        <back::Backend as Backend>::GraphicsPipeline,
        Vec<<back::Backend as Backend>::Framebuffer>,
        pool::CommandPool<back::Backend, Graphics>,
        Vec<command::Submit<back::Backend, Graphics, command::MultiShot, command::Primary>>,
        Vec<<back::Backend as Backend>::Semaphore>,
        Vec<<back::Backend as Backend>::Semaphore>,
        Vec<<back::Backend as Backend>::Fence>,
    ) {
        let instance = HelloTriangleApplication::create_instance();
        let mut adapter = HelloTriangleApplication::pick_adapter(&instance);
        let mut surface = HelloTriangleApplication::create_surface(&instance, window);
        let (device, command_queues, queue_type, qf_id) =
            HelloTriangleApplication::create_device_with_graphics_queues(&mut adapter, &surface);
        let (swapchain, extent, backbuffer, format) =
            HelloTriangleApplication::create_swap_chain(&adapter, &device, &mut surface, None);
        let frame_images =
            HelloTriangleApplication::create_image_views(backbuffer, format, &device);
        let render_pass = HelloTriangleApplication::create_render_pass(&device, Some(format));
        let (ds_layouts, pipeline_layout, gfx_pipeline) =
            HelloTriangleApplication::create_graphics_pipeline(&device, extent, &render_pass);
        let swapchain_framebuffers = HelloTriangleApplication::create_framebuffers(
            &device,
            &render_pass,
            &frame_images,
            extent,
        );
        let mut command_pool =
            HelloTriangleApplication::create_command_pool(&device, queue_type, qf_id);
        let submission_command_buffers = HelloTriangleApplication::create_command_buffers(
            &mut command_pool,
            &render_pass,
            &swapchain_framebuffers,
            extent,
            &gfx_pipeline,
        );
        let (image_available_semaphores, render_finished_semaphores, in_flight_fences) =
            HelloTriangleApplication::create_sync_objects(&device);

        (
            instance,
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
            submission_command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
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
        queue::QueueType,
        queue::family::QueueFamilyId,
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

        (device, command_queues, family.queue_type(), family.id())
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
        let extent = swap_config.extent.clone();
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
            // OpenGL case, where backbuffer is a framebuffer, not implemented currently
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
        render_pass: &<back::Backend as Backend>::RenderPass,
    ) -> (
        Vec<<back::Backend as Backend>::DescriptorSetLayout>,
        <back::Backend as Backend>::PipelineLayout,
        <back::Backend as Backend>::GraphicsPipeline,
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

            let shaders = pso::GraphicsShaderSet {
                vertex: vs_entry,
                hull: None,
                domain: None,
                geometry: None,
                fragment: Some(fs_entry),
            };

            let rasterizer = pso::Rasterizer {
                depth_clamping: false,
                polygon_mode: pso::PolygonMode::Fill,
                cull_face: <pso::Face>::BACK,
                front_face: pso::FrontFace::Clockwise,
                depth_bias: None,
                conservative: false,
            };

            let vertex_buffers: Vec<pso::VertexBufferDesc> = Vec::new();
            let attributes: Vec<pso::AttributeDesc> = Vec::new();

            let input_assembler = pso::InputAssemblerDesc::new(Primitive::TriangleList);

            let blender = {
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

            let depth_stencil = pso::DepthStencilDesc {
                depth: pso::DepthTest::Off,
                depth_bounds: false,
                stencil: pso::StencilTest::Off,
            };

            let multisampling: Option<pso::Multisampling> = None;

            let baked_states = pso::BakedStates {
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

            let subpass = pass::Subpass {
                index: 0,
                main_pass: render_pass,
            };

            let flags = pso::PipelineCreationFlags::empty();

            let parent = pso::BasePipeline::None;

            let gfx_pipeline = {
                let desc = pso::GraphicsPipelineDesc {
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
        device: &<back::Backend as Backend>::Device,
        render_pass: &<back::Backend as Backend>::RenderPass,
        frame_images: &Vec<(
            <back::Backend as Backend>::Image,
            <back::Backend as Backend>::ImageView,
        )>,
        extent: window::Extent2D,
    ) -> Vec<<back::Backend as Backend>::Framebuffer> {
        let mut swapchain_framebuffers: Vec<<back::Backend as Backend>::Framebuffer> = Vec::new();

        for (_, image_view) in frame_images.iter() {
            swapchain_framebuffers.push(
                device
                    .create_framebuffer(
                        render_pass,
                        vec![image_view],
                        image::Extent {
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
        command_pool: &'a mut pool::CommandPool<back::Backend, Graphics>,
        render_pass: &<back::Backend as Backend>::RenderPass,
        framebuffers: &Vec<<back::Backend as Backend>::Framebuffer>,
        extent: window::Extent2D,
        pipeline: &<back::Backend as Backend>::GraphicsPipeline,
    ) -> Vec<command::Submit<back::Backend, Graphics, command::MultiShot, command::Primary>> {
        // reserve (allocate memory for) num number of primary command buffers
        command_pool.reserve(framebuffers.iter().count());

        let mut submission_command_buffers: Vec<
            command::Submit<back::Backend, Graphics, command::MultiShot, command::Primary>,
        > = Vec::new();

        for fb in framebuffers.iter() {
            // command buffer will be returned in 'recording' state
            // Shot: how many times a command buffer can be submitted; we want MultiShot (allow submission multiple times)
            // Level: command buffer type (primary or secondary)
            // allow_pending_resubmit is set to true, as we want to allow simultaneous use per vulkan-tutorial
            let mut command_buffer: command::CommandBuffer<
                back::Backend,
                Graphics,
                command::MultiShot,
                command::Primary,
            > = command_pool.acquire_command_buffer(true);

            command_buffer.bind_graphics_pipeline(pipeline);
            {
                // begin render pass
                let render_area = pso::Rect {
                    x: 0,
                    y: 0,
                    w: extent.width as _,
                    h: extent.height as _,
                };
                let clear_values = vec![command::ClearValue::Color(command::ClearColor::Float([
                    0.0, 0.0, 0.0, 4.0,
                ]))];

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
        device: &<back::Backend as Backend>::Device,
        queue_type: queue::QueueType,
        qf_id: queue::family::QueueFamilyId,
    ) -> pool::CommandPool<back::Backend, Graphics> {
        // raw command pool: a thin wrapper around command pools
        // strongly typed command pool: a safe wrapper around command pools, which ensures that only one command buffer is recorded at the same time from the current queue
        let raw_command_pool =
            device.create_command_pool(qf_id, pool::CommandPoolCreateFlags::empty());

        // safety check necessary before creating a strongly typed command pool
        assert_eq!(Graphics::supported_by(queue_type), true);
        unsafe { pool::CommandPool::new(raw_command_pool) }
    }

    fn draw_frame(
        device: &<back::Backend as Backend>::Device,
        command_queues: &mut Vec<queue::CommandQueue<back::Backend, Graphics>>,
        swapchain: &mut <back::Backend as Backend>::Swapchain,
        submission_command_buffers: &Vec<
            command::Submit<back::Backend, Graphics, command::MultiShot, command::Primary>,
        >,
        image_available_semaphore: &<back::Backend as Backend>::Semaphore,
        render_finished_semaphore: &<back::Backend as Backend>::Semaphore,
        in_flight_fence: &<back::Backend as Backend>::Fence,
    ) {
        device.wait_for_fence(in_flight_fence, std::u64::MAX);
        device.reset_fence(in_flight_fence);

        let image_index = swapchain
            .acquire_image(
                std::u64::MAX,
                window::FrameSync::Semaphore(image_available_semaphore),
            ).expect("could not acquire image!");

        let submission = queue::submission::Submission::new()
            .wait_on(&[(
                image_available_semaphore,
                pso::PipelineStage::COLOR_ATTACHMENT_OUTPUT,
            )]).signal(vec![render_finished_semaphore])
            .submit(Some(&submission_command_buffers[image_index as usize]));

        // recall we only made one queue
        command_queues[0].submit(submission, Some(in_flight_fence));

        swapchain
            .present(
                &mut command_queues[0],
                image_index,
                vec![render_finished_semaphore],
            ).expect("presentation failed!");
    }

    fn create_sync_objects(
        device: &<back::Backend as Backend>::Device,
    ) -> (
        Vec<<back::Backend as Backend>::Semaphore>,
        Vec<<back::Backend as Backend>::Semaphore>,
        Vec<<back::Backend as Backend>::Fence>,
    ) {
        let mut image_available_semaphores: Vec<<back::Backend as Backend>::Semaphore> = Vec::new();
        let mut render_finished_semaphores: Vec<<back::Backend as Backend>::Semaphore> = Vec::new();
        let mut in_flight_fences: Vec<<back::Backend as Backend>::Fence> = Vec::new();

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            image_available_semaphores.push(device.create_semaphore());
            render_finished_semaphores.push(device.create_semaphore());
            in_flight_fences.push(device.create_fence(true));
        }

        (
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
        )
    }

    fn main_loop(&mut self) {
        let mut current_frame: usize = 0;

        let mut events_loop = self
            .events_loop
            .take()
            .expect("events_loop does not exist!");
        events_loop.run_forever(|event| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                self.device.wait_idle().expect("Queues are not going idle!");
                ControlFlow::Break
            }
            _ => {
                match (
                    &mut self.swapchain,
                    &self.submission_command_buffers,
                    &self.image_available_semaphores,
                    &self.render_finished_semaphores,
                    &self.in_flight_fences,
                ) {
                    (
                        Some(swapchain),
                        Some(submission_command_buffers),
                        Some(image_available_semaphores),
                        Some(render_finished_semaphores),
                        Some(in_flight_fences),
                    ) => {
                        HelloTriangleApplication::draw_frame(
                            &self.device,
                            &mut self.command_queues,
                            swapchain,
                            &submission_command_buffers,
                            &image_available_semaphores[current_frame],
                            &render_finished_semaphores[current_frame],
                            &in_flight_fences[current_frame],
                        );
                    }
                    _ => {
                        panic!("One of requisite arguments for draw_frame does not exist!");
                    }
                }

                current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;;
                ControlFlow::Continue
            }
        });
        self.events_loop = Some(events_loop);
    }

    pub fn run(&mut self) {
        self.main_loop();
    }
}

impl Drop for HelloTriangleApplication {
    fn drop(&mut self) {
        match self.in_flight_fences.take() {
            Some(fences) => {
                for f in fences {
                    self.device.destroy_fence(f);
                }
            }
            _ => {}
        }

        match self.render_finished_semaphores.take() {
            Some(sems) => {
                for s in sems {
                    self.device.destroy_semaphore(s);
                }
            }
            _ => {}
        }

        match self.image_available_semaphores.take() {
            Some(sems) => {
                for s in sems {
                    self.device.destroy_semaphore(s);
                }
            }
            _ => {}
        }

        match self.command_pool.take() {
            Some(cp) => {
                self.device.destroy_command_pool(cp.into_raw());
            }
            _ => {}
        }

        match self.swapchain_framebuffers.take() {
            Some(fbs) => {
                for fb in fbs {
                    self.device.destroy_framebuffer(fb);
                }
            }
            _ => {}
        }

        match self.gfx_pipeline.take() {
            Some(gp) => {
                self.device.destroy_graphics_pipeline(gp);
            }
            _ => {}
        }

        match self.descriptor_set_layouts.take() {
            Some(dsls) => {
                for dsl in dsls {
                    self.device.destroy_descriptor_set_layout(dsl);
                }
            }
            _ => {}
        }

        match self.pipeline_layout.take() {
            Some(pl) => {
                self.device.destroy_pipeline_layout(pl);
            }
            _ => {}
        }

        match self.render_pass.take() {
            Some(rp) => {
                self.device.destroy_render_pass(rp);
            }
            _ => {}
        }

        match self.frame_images.take() {
            Some(fis) => {
                for (_, v) in fis {
                    self.device.destroy_image_view(v);
                }
            }
            _ => {}
        }

        match self.swapchain.take() {
            Some(swapchain) => {
                self.device.destroy_swapchain(swapchain);
            }
            _ => {}
        }
    }
}
