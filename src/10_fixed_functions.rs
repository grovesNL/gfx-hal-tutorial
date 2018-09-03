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

use hal::{Capability, Device, Instance, PhysicalDevice, QueueFamily, Surface, SwapchainConfig};
use std::io::Read;

static WINDOW_NAME: &str = "10_fixed_functions";

fn main() {
    env_logger::init();

    let (window, events_loop) = init_window();

    let (
        _instance,
        device,
        swapchain,
        _surface,
        frame_images,
        descriptor_set_layouts,
        pipeline_layout,
    ) = init_hal(&window);
    main_loop(events_loop);

    clean_up(
        device,
        frame_images,
        swapchain,
        descriptor_set_layouts,
        pipeline_layout,
    );
}

fn create_graphics_pipeline(
    device: &<back::Backend as hal::Backend>::Device,
    extent: hal::window::Extent2D,
) -> (
    Vec<<back::Backend as hal::Backend>::DescriptorSetLayout>,
    <back::Backend as hal::Backend>::PipelineLayout,
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

        // no need to set up vertex input format, as it is hardcoded
        let vertex_buffers: Vec<hal::pso::VertexBufferDesc> = Vec::new();
        let attributes: Vec<hal::pso::AttributeDesc> = Vec::new();

        let input_assembler = hal::pso::InputAssemblerDesc::new(hal::Primitive::TriangleList);

        // implements optional blending description provided in vulkan-tutorial
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

        // viewports and scissors
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

        // with HAL, user only needs to specify whether overall state is static or dynamic

        // pipeline layout
        let bindings = Vec::<hal::pso::DescriptorSetLayoutBinding>::new();
        let immutable_samplers = Vec::<<back::Backend as hal::Backend>::Sampler>::new();
        let ds_layouts: Vec<<back::Backend as hal::Backend>::DescriptorSetLayout> =
            vec![device.create_descriptor_set_layout(bindings, immutable_samplers)];
        let push_constants = Vec::<(hal::pso::ShaderStageFlags, std::ops::Range<u32>)>::new();
        let layout = device.create_pipeline_layout(&ds_layouts, push_constants);

        // our goal is to fill out this entire struct
        //    let desc = hal::pso::GraphicsPipelineDesc {
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

fn create_image_views(
    backbuffer: hal::Backbuffer<back::Backend>,
    format: hal::format::Format,
    device: &<back::Backend as hal::Backend>::Device,
) -> Vec<(
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
        _ => unimplemented!(),
    }
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

fn create_surface(
    instance: &back::Instance,
    window: &winit::Window,
) -> <back::Backend as hal::Backend>::Surface {
    instance.create_surface(window)
}

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

fn init_hal(
    window: &winit::Window,
) -> (
    back::Instance,
    <back::Backend as hal::Backend>::Device,
    <back::Backend as hal::Backend>::Swapchain,
    <back::Backend as hal::Backend>::Surface,
    Vec<(
        <back::Backend as hal::Backend>::Image,
        <back::Backend as hal::Backend>::ImageView,
    )>,
    Vec<<back::Backend as hal::Backend>::DescriptorSetLayout>,
    <back::Backend as hal::Backend>::PipelineLayout,
) {
    let instance = create_instance();
    let mut surface = create_surface(&instance, window);
    let mut adapter = pick_adapter(&instance);
    let (device, _command_queues) = create_device_with_graphics_queues(&mut adapter, &surface);
    let (swapchain, extent, backbuffer, format) =
        create_swap_chain(&adapter, &device, &mut surface, None);
    let frame_images = create_image_views(backbuffer, format, &device);
    let (descriptor_set_layouts, pipeline_layout) = create_graphics_pipeline(&device, extent);
    (
        instance,
        device,
        swapchain,
        surface,
        frame_images,
        descriptor_set_layouts,
        pipeline_layout,
    )
}

fn clean_up(
    device: <back::Backend as hal::Backend>::Device,
    frame_images: Vec<(
        <back::Backend as hal::Backend>::Image,
        <back::Backend as hal::Backend>::ImageView,
    )>,
    swapchain: <back::Backend as hal::Backend>::Swapchain,
    descriptor_set_layouts: Vec<<back::Backend as hal::Backend>::DescriptorSetLayout>,
    pipeline_layout: <back::Backend as hal::Backend>::PipelineLayout,
) {
    for dsl in descriptor_set_layouts.into_iter() {
        device.destroy_descriptor_set_layout(dsl);
    }

    device.destroy_pipeline_layout(pipeline_layout);

    for (_, image_view) in frame_images.into_iter() {
        device.destroy_image_view(image_view);
    }

    device.destroy_swapchain(swapchain);
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
