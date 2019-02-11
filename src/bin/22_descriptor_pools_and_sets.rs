#[macro_use]
extern crate vulkano;
extern crate vulkano_win;
extern crate winit;

use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use std::time::Instant;

use winit::{EventsLoop, WindowBuilder, Window, dpi::LogicalSize, Event, WindowEvent};
use vulkano_win::VkSurfaceBuild;

use vulkano::instance::{
    Instance,
    InstanceExtensions,
    ApplicationInfo,
    Version,
    layers_list,
    PhysicalDevice,
};
use vulkano::instance::debug::{DebugCallback, MessageTypes};
use vulkano::device::{Device, DeviceExtensions, Queue, Features};
use vulkano::swapchain::{
    Surface,
    Capabilities,
    ColorSpace,
    SupportedPresentModes,
    PresentMode,
    Swapchain,
    CompositeAlpha,
    acquire_next_image,
    AcquireError,
};
use vulkano::format::Format;
use vulkano::image::{ImageUsage, swapchain::SwapchainImage};
use vulkano::sync::{self, SharingMode, GpuFuture};
use vulkano::pipeline::{
    GraphicsPipeline,
    GraphicsPipelineAbstract,
    viewport::Viewport,
};
use vulkano::framebuffer::{
    RenderPassAbstract,
    Subpass,
    FramebufferAbstract,
    Framebuffer,
};
use vulkano::command_buffer::{
    AutoCommandBuffer,
    AutoCommandBufferBuilder,
    DynamicState,
};
use vulkano::buffer::{
    immutable::ImmutableBuffer,
    BufferUsage,
    BufferAccess,
    TypedBufferAccess,
    CpuAccessibleBuffer
};
use vulkano::descriptor::descriptor_set::{
    FixedSizeDescriptorSetsPool,
    FixedSizeDescriptorSet,
    PersistentDescriptorSetBuf
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const VALIDATION_LAYERS: &[&str] =  &[
    "VK_LAYER_LUNARG_standard_validation"
];

/// Required device extensions
fn device_extensions() -> DeviceExtensions {
    DeviceExtensions {
        khr_swapchain: true,
        .. vulkano::device::DeviceExtensions::none()
    }
}

#[cfg(all(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

struct QueueFamilyIndices {
    graphics_family: i32,
    present_family: i32,
}
impl QueueFamilyIndices {
    fn new() -> Self {
        Self { graphics_family: -1, present_family: -1 }
    }

    fn is_complete(&self) -> bool {
        self.graphics_family >= 0 && self.present_family >= 0
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 3],
}
impl Vertex {
    fn new(pos: [f32; 2], color: [f32; 3]) -> Self {
        Self { pos, color }
    }
}
impl_vertex!(Vertex, pos, color);

#[allow(dead_code)]
#[derive(Copy, Clone)]
struct UniformBufferObject {
    model: glm::Mat4,
    view: glm::Mat4,
    proj: glm::Mat4,
}

fn vertices() -> [Vertex; 4] {
    [
        Vertex::new([-0.5, -0.5], [1.0, 0.0, 0.0]),
        Vertex::new([0.5, -0.5], [0.0, 1.0, 0.0]),
        Vertex::new([0.5, 0.5], [0.0, 0.0, 1.0]),
        Vertex::new([-0.5, 0.5], [1.0, 1.0, 1.0])
    ]
}

fn indices() -> [u16; 6] {
    [0, 1, 2, 2, 3, 0]
}

type DescriptorSetResources = ((), PersistentDescriptorSetBuf<Arc<CpuAccessibleBuffer<UniformBufferObject>>>);

struct HelloTriangleApplication {
    instance: Arc<Instance>,
    #[allow(unused)]
    debug_callback: Option<DebugCallback>,

    events_loop: EventsLoop,
    surface: Arc<Surface<Window>>,

    physical_device_index: usize, // can't store PhysicalDevice directly (lifetime issues)
    device: Arc<Device>,

    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,

    swap_chain: Arc<Swapchain<Window>>,
    swap_chain_images: Vec<Arc<SwapchainImage<Window>>>,

    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    graphics_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,

    swap_chain_framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,

    vertex_buffer: Arc<BufferAccess + Send + Sync>,
    index_buffer: Arc<TypedBufferAccess<Content=[u16]> + Send + Sync>,
    uniform_buffers: Vec<Arc<CpuAccessibleBuffer<UniformBufferObject>>>,

    descriptor_sets: Vec<Arc<FixedSizeDescriptorSet<Arc<GraphicsPipelineAbstract + Send + Sync>, DescriptorSetResources>>>,

    command_buffers: Vec<Arc<AutoCommandBuffer>>,

    previous_frame_end: Option<Box<GpuFuture>>,
    recreate_swap_chain: bool,

    start_time: Instant,
}

impl HelloTriangleApplication {
    pub fn initialize() -> Self {
        let instance = Self::create_instance();
        let debug_callback = Self::setup_debug_callback(&instance);
        let (events_loop, surface) = Self::create_surface(&instance);

        let physical_device_index = Self::pick_physical_device(&instance, &surface);
        let (device, graphics_queue, present_queue) = Self::create_logical_device(
            &instance, &surface, physical_device_index);

        let (swap_chain, swap_chain_images) = Self::create_swap_chain(&instance, &surface, physical_device_index,
            &device, &graphics_queue, &present_queue, None);

        let render_pass = Self::create_render_pass(&device, swap_chain.format());

        let graphics_pipeline = Self::create_graphics_pipeline(&device, swap_chain.dimensions(), &render_pass);

        let swap_chain_framebuffers = Self::create_framebuffers(&swap_chain_images, &render_pass);

        let start_time = Instant::now();

        let vertex_buffer = Self::create_vertex_buffer(&graphics_queue);
        let index_buffer = Self::create_index_buffer(&graphics_queue);
        let uniform_buffers = Self::create_uniform_buffers(&device, swap_chain_images.len(), start_time, swap_chain.dimensions());

        let descriptor_sets_pool = Self::create_descriptor_pool(&graphics_pipeline);
        let descriptor_sets = Self::create_descriptor_sets(&descriptor_sets_pool, &uniform_buffers);

        let previous_frame_end = Some(Self::create_sync_objects(&device));

        let mut app = Self {
            instance,
            debug_callback,

            events_loop,
            surface,

            physical_device_index,
            device,

            graphics_queue,
            present_queue,

            swap_chain,
            swap_chain_images,

            render_pass,
            graphics_pipeline,

            swap_chain_framebuffers,

            vertex_buffer,
            index_buffer,
            uniform_buffers,

            descriptor_sets,

            command_buffers: vec![],

            previous_frame_end,
            recreate_swap_chain: false,

            start_time
        };

        app.create_command_buffers();
        app
    }

    fn create_instance() -> Arc<Instance> {
        if ENABLE_VALIDATION_LAYERS && !Self::check_validation_layer_support() {
            println!("Validation layers requested, but not available!")
        }

        let supported_extensions = InstanceExtensions::supported_by_core()
            .expect("failed to retrieve supported extensions");
        println!("Supported extensions: {:?}", supported_extensions);

        let app_info = ApplicationInfo {
            application_name: Some("Hello Triangle".into()),
            application_version: Some(Version { major: 1, minor: 0, patch: 0 }),
            engine_name: Some("No Engine".into()),
            engine_version: Some(Version { major: 1, minor: 0, patch: 0 }),
        };

        let required_extensions = Self::get_required_extensions();

        if ENABLE_VALIDATION_LAYERS && Self::check_validation_layer_support() {
            Instance::new(Some(&app_info), &required_extensions, VALIDATION_LAYERS.iter().cloned())
                .expect("failed to create Vulkan instance")
        } else {
            Instance::new(Some(&app_info), &required_extensions, None)
                .expect("failed to create Vulkan instance")
        }
    }

    fn check_validation_layer_support() -> bool {
        let layers: Vec<_> = layers_list().unwrap().map(|l| l.name().to_owned()).collect();
        VALIDATION_LAYERS.iter()
            .all(|layer_name| layers.contains(&layer_name.to_string()))
    }

    fn get_required_extensions() -> InstanceExtensions {
        let mut extensions = vulkano_win::required_extensions();
        if ENABLE_VALIDATION_LAYERS {
            // TODO!: this should be ext_debug_utils (_report is deprecated), but that doesn't exist yet in vulkano
            extensions.ext_debug_report = true;
        }

        extensions
    }

    fn setup_debug_callback(instance: &Arc<Instance>) -> Option<DebugCallback> {
        if !ENABLE_VALIDATION_LAYERS  {
            return None;
        }

        let msg_types = MessageTypes {
            error: true,
            warning: true,
            performance_warning: true,
            information: false,
            debug: true,
        };
        DebugCallback::new(&instance, msg_types, |msg| {
            println!("validation layer: {:?}", msg.description);
        }).ok()
    }

    fn pick_physical_device(instance: &Arc<Instance>, surface: &Arc<Surface<Window>>) -> usize {
        PhysicalDevice::enumerate(&instance)
            .position(|device| Self::is_device_suitable(surface, &device))
            .expect("failed to find a suitable GPU!")
    }

    fn is_device_suitable(surface: &Arc<Surface<Window>>, device: &PhysicalDevice) -> bool {
        let indices = Self::find_queue_families(surface, device);
        let extensions_supported = Self::check_device_extension_support(device);

        let swap_chain_adequate = if extensions_supported {
                let capabilities = surface.capabilities(*device)
                    .expect("failed to get surface capabilities");
                !capabilities.supported_formats.is_empty() &&
                    capabilities.present_modes.iter().next().is_some()
            } else {
                false
            };

        indices.is_complete() && extensions_supported && swap_chain_adequate
    }

    fn check_device_extension_support(device: &PhysicalDevice) -> bool {
        let available_extensions = DeviceExtensions::supported_by_device(*device);
        let device_extensions = device_extensions();
        available_extensions.intersection(&device_extensions) == device_extensions
    }

    fn choose_swap_surface_format(available_formats: &[(Format, ColorSpace)]) -> (Format, ColorSpace) {
        // NOTE: the 'preferred format' mentioned in the tutorial doesn't seem to be
        // queryable in Vulkano (no VK_FORMAT_UNDEFINED enum)
        *available_formats.iter()
            .find(|(format, color_space)|
                *format == Format::B8G8R8A8Unorm && *color_space == ColorSpace::SrgbNonLinear
            )
            .unwrap_or_else(|| &available_formats[0])
    }

    fn choose_swap_present_mode(available_present_modes: SupportedPresentModes) -> PresentMode {
        if available_present_modes.mailbox {
            PresentMode::Mailbox
        } else if available_present_modes.immediate {
            PresentMode::Immediate
        } else {
            PresentMode::Fifo
        }
    }

    fn choose_swap_extent(capabilities: &Capabilities) -> [u32; 2] {
        if let Some(current_extent) = capabilities.current_extent {
            return current_extent
        } else {
            let mut actual_extent = [WIDTH, HEIGHT];
            actual_extent[0] = capabilities.min_image_extent[0]
                .max(capabilities.max_image_extent[0].min(actual_extent[0]));
            actual_extent[1] = capabilities.min_image_extent[1]
                .max(capabilities.max_image_extent[1].min(actual_extent[1]));
            actual_extent
        }
    }

    fn create_swap_chain(
        instance: &Arc<Instance>,
        surface: &Arc<Surface<Window>>,
        physical_device_index: usize,
        device: &Arc<Device>,
        graphics_queue: &Arc<Queue>,
        present_queue: &Arc<Queue>,
        old_swapchain: Option<Arc<Swapchain<Window>>>,
    ) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
        let physical_device = PhysicalDevice::from_index(&instance, physical_device_index).unwrap();
        let capabilities = surface.capabilities(physical_device)
            .expect("failed to get surface capabilities");

        let surface_format = Self::choose_swap_surface_format(&capabilities.supported_formats);
        let present_mode = Self::choose_swap_present_mode(capabilities.present_modes);
        let extent = Self::choose_swap_extent(&capabilities);

        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count.is_some() && image_count > capabilities.max_image_count.unwrap() {
            image_count = capabilities.max_image_count.unwrap();
        }

        let image_usage = ImageUsage {
            color_attachment: true,
            .. ImageUsage::none()
        };

        let indices = Self::find_queue_families(&surface, &physical_device);

        let sharing: SharingMode = if indices.graphics_family != indices.present_family {
            vec![graphics_queue, present_queue].as_slice().into()
        } else {
            graphics_queue.into()
        };

        let (swap_chain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            image_count,
            surface_format.0, // TODO: color space?
            extent,
            1, // layers
            image_usage,
            sharing,
            capabilities.current_transform,
            CompositeAlpha::Opaque,
            present_mode,
            true, // clipped
            old_swapchain.as_ref()
        ).expect("failed to create swap chain!");

        (swap_chain, images)
    }

    fn create_render_pass(device: &Arc<Device>, color_format: Format) -> Arc<RenderPassAbstract + Send + Sync> {
        Arc::new(single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: color_format,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap())
    }

    fn create_graphics_pipeline(
        device: &Arc<Device>,
        swap_chain_extent: [u32; 2],
        render_pass: &Arc<RenderPassAbstract + Send + Sync>,
    ) -> Arc<GraphicsPipelineAbstract + Send + Sync> {
        mod vertex_shader {
            vulkano_shaders::shader! {
               ty: "vertex",
               path: "src/bin/21_shader_uniformbuffer.vert"
            }
        }

        mod fragment_shader {
            vulkano_shaders::shader! {
                ty: "fragment",
                path: "src/bin/21_shader_uniformbuffer.frag"
            }
        }

        let vert_shader_module = vertex_shader::Shader::load(device.clone())
            .expect("failed to create vertex shader module!");
        let frag_shader_module = fragment_shader::Shader::load(device.clone())
            .expect("failed to create fragment shader module!");

        let dimensions = [swap_chain_extent[0] as f32, swap_chain_extent[1] as f32];
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions,
            depth_range: 0.0 .. 1.0,
        };

        Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vert_shader_module.main_entry_point(), ())
            .triangle_list()
            .primitive_restart(false)
            .viewports(vec![viewport]) // NOTE: also sets scissor to cover whole viewport
            .fragment_shader(frag_shader_module.main_entry_point(), ())
            .depth_clamp(false)
            // NOTE: there's an outcommented .rasterizer_discard() in Vulkano...
            .polygon_mode_fill() // = default
            .line_width(1.0) // = default
            .cull_mode_back()
            .front_face_counter_clockwise()
            // NOTE: no depth_bias here, but on pipeline::raster::Rasterization
            .blend_pass_through() // = default
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap()
        )
    }

    fn create_framebuffers(
        swap_chain_images: &[Arc<SwapchainImage<Window>>],
        render_pass: &Arc<RenderPassAbstract + Send + Sync>
    ) -> Vec<Arc<FramebufferAbstract + Send + Sync>> {
        swap_chain_images.iter()
            .map(|image| {
                let fba: Arc<FramebufferAbstract + Send + Sync> = Arc::new(Framebuffer::start(render_pass.clone())
                    .add(image.clone()).unwrap()
                    .build().unwrap());
                fba
            }
        ).collect::<Vec<_>>()
    }

    fn create_vertex_buffer(graphics_queue: &Arc<Queue>) -> Arc<BufferAccess + Send + Sync> {
        let (buffer, future) = ImmutableBuffer::from_iter(
            vertices().iter().cloned(), BufferUsage::vertex_buffer(),
            graphics_queue.clone())
            .unwrap();
        future.flush().unwrap();
        buffer
    }

    fn create_index_buffer(graphics_queue: &Arc<Queue>) -> Arc<TypedBufferAccess<Content=[u16]> + Send + Sync> {
        let (buffer, future) = ImmutableBuffer::from_iter(
            indices().iter().cloned(), BufferUsage::index_buffer(),
            graphics_queue.clone())
            .unwrap();
        future.flush().unwrap();
        buffer
    }

    fn create_uniform_buffers(
        device: &Arc<Device>,
        num_buffers: usize,
        start_time: Instant,
        dimensions_u32: [u32; 2]
    ) -> Vec<Arc<CpuAccessibleBuffer<UniformBufferObject>>> {
        let mut buffers = Vec::new();

        let dimensions = [dimensions_u32[0] as f32, dimensions_u32[1] as f32];

        let uniform_buffer = Self::update_uniform_buffer(start_time, dimensions);

        for _ in 0..num_buffers {
            let buffer = CpuAccessibleBuffer::from_data(
                device.clone(),
                BufferUsage::uniform_buffer_transfer_destination(),
                uniform_buffer,
            ).unwrap();

            buffers.push(buffer);
        }

        buffers
    }

    fn create_descriptor_pool(graphics_pipeline: &Arc<GraphicsPipelineAbstract + Send + Sync>)
        -> Arc<Mutex<FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Send + Sync>>>>
    {
        Arc::new(
            Mutex::new(
                FixedSizeDescriptorSetsPool::new(graphics_pipeline.clone(), 0)
            )
        )
    }

    fn create_descriptor_sets(
        pool: &Arc<Mutex<FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Send + Sync>>>>,
        uniform_buffers: &[Arc<CpuAccessibleBuffer<UniformBufferObject>>],
    ) -> Vec<Arc<FixedSizeDescriptorSet<Arc<GraphicsPipelineAbstract + Send + Sync>, DescriptorSetResources>>>
    {
        uniform_buffers
            .iter()
            .map(|uniform_buffer|
                Arc::new(
                    pool
                        .lock()
                        .unwrap()
                        .next()
                        .add_buffer(uniform_buffer.clone())
                        .unwrap()
                        .build()
                        .unwrap()
                )
            )
            .collect()
    }

    fn create_command_buffers(&mut self) {
        let queue_family = self.graphics_queue.family();
        let dimensions = [self.swap_chain.dimensions()[0] as f32, self.swap_chain.dimensions()[1] as f32];

        self.command_buffers = self.swap_chain_framebuffers
            .iter()
            .enumerate()
            .map(|(i, framebuffer)| {
                Arc::new(AutoCommandBufferBuilder::primary_simultaneous_use(self.device.clone(), queue_family)
                    .unwrap()
                    .update_buffer(self.uniform_buffers[i].clone(), Self::update_uniform_buffer(self.start_time, dimensions))
                    .unwrap()
                    .begin_render_pass(framebuffer.clone(), false, vec![[0.0, 0.0, 0.0, 1.0].into()])
                    .unwrap()
                    .draw_indexed(
                        self.graphics_pipeline.clone(),
                        &DynamicState::none(),
                        vec![self.vertex_buffer.clone()],
                        self.index_buffer.clone(),
                        self.descriptor_sets[i].clone(),
                        ())
                    .unwrap()
                    .end_render_pass()
                    .unwrap()
                    .build()
                    .unwrap())
            })
            .collect();
    }

    fn create_sync_objects(device: &Arc<Device>) -> Box<GpuFuture> {
        Box::new(sync::now(device.clone())) as Box<GpuFuture>
    }

    fn find_queue_families(surface: &Arc<Surface<Window>>, device: &PhysicalDevice) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices::new();
        // TODO: replace index with id to simplify?
        for (i, queue_family) in device.queue_families().enumerate() {
            if queue_family.supports_graphics() {
                indices.graphics_family = i as i32;
            }

            if surface.is_supported(queue_family).unwrap() {
                indices.present_family = i as i32;
            }

            if indices.is_complete() {
                break;
            }
        }

        indices
    }

    fn create_logical_device(
        instance: &Arc<Instance>,
        surface: &Arc<Surface<Window>>,
        physical_device_index: usize,
    ) -> (Arc<Device>, Arc<Queue>, Arc<Queue>) {
        let physical_device = PhysicalDevice::from_index(&instance, physical_device_index).unwrap();
        let indices = Self::find_queue_families(&surface, &physical_device);

        let families = [indices.graphics_family, indices.present_family];
        use std::iter::FromIterator;
        let unique_queue_families: HashSet<&i32> = HashSet::from_iter(families.iter());

        let queue_priority = 1.0;
        let queue_families = unique_queue_families.iter().map(|i| {
            (physical_device.queue_families().nth(**i as usize).unwrap(), queue_priority)
        });

        // NOTE: the tutorial recommends passing the validation layers as well
        // for legacy reasons (if ENABLE_VALIDATION_LAYERS is true). Vulkano handles that
        // for us internally.

        let (device, mut queues) = Device::new(physical_device, &Features::none(),
            &device_extensions(), queue_families)
            .expect("failed to create logical device!");

        let graphics_queue = queues.next().unwrap();
        let present_queue = queues.next().unwrap_or_else(|| graphics_queue.clone());

        (device, graphics_queue, present_queue)
    }

    fn create_surface(instance: &Arc<Instance>) -> (EventsLoop, Arc<Surface<Window>>) {
        let events_loop = EventsLoop::new();
        let surface = WindowBuilder::new()
            .with_title("Vulkan")
            .with_dimensions(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
            .build_vk_surface(&events_loop, instance.clone())
            .expect("failed to create window surface!");
        (events_loop, surface)
    }

    #[allow(unused)]
    fn main_loop(&mut self) {
        loop {
            self.create_command_buffers();
            self.draw_frame();

            let mut done = false;
            self.events_loop.poll_events(|ev| {
                if let Event::WindowEvent { event: WindowEvent::CloseRequested, .. } = ev {
                    done = true
                }
            });
            if done {
                return;
            }
        }
    }

    fn draw_frame(&mut self) {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swap_chain {
            self.recreate_swap_chain();
            self.recreate_swap_chain = false;
        }

        let (image_index, acquire_future) = match acquire_next_image(self.swap_chain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                self.recreate_swap_chain = true;
                return;
            },
            Err(err) => panic!("{:?}", err)
        };

        let command_buffer = self.command_buffers[image_index].clone();

        let future = self.previous_frame_end.take().unwrap()
            .join(acquire_future)
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.present_queue.clone(), self.swap_chain.clone(), image_index)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(Box::new(future) as Box<_>);
            }
            Err(vulkano::sync::FlushError::OutOfDate) => {
                self.recreate_swap_chain = true;
                self.previous_frame_end
                    = Some(Box::new(vulkano::sync::now(self.device.clone())) as Box<_>);
            }
            Err(e) => {
                println!("{:?}", e);
                self.previous_frame_end
                    = Some(Box::new(vulkano::sync::now(self.device.clone())) as Box<_>);
            }
        }
    }

    fn update_uniform_buffer(start_time: Instant, dimensions: [f32; 2]) -> UniformBufferObject {
        let duration = Instant::now().duration_since(start_time);
        let elapsed = (duration.as_secs() * 1000) + u64::from(duration.subsec_millis());

        let identity_matrix = glm::mat4(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );

        let model = glm::ext::rotate(&identity_matrix, (elapsed as f32) * glm::radians(0.180), glm::vec3(0.0, 0.0, 1.00));

        let view = glm::ext::look_at(
            glm::vec3(2.0, 2.0, 2.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, 0.0, 1.0)
        );
        let mut proj = glm::ext::perspective(
            glm::radians(45.0,),
            dimensions[0] as f32 / dimensions[1] as f32,
            0.1,
            10.0
        );

        proj.c1.y *= -1.0;

        UniformBufferObject { model, view, proj }
    }

    fn recreate_swap_chain(&mut self) {
        let (swap_chain, images) = Self::create_swap_chain(&self.instance, &self.surface, self.physical_device_index,
            &self.device, &self.graphics_queue, &self.present_queue, Some(self.swap_chain.clone()));
        self.swap_chain = swap_chain;
        self.swap_chain_images = images;

        self.render_pass = Self::create_render_pass(&self.device, self.swap_chain.format());
        self.graphics_pipeline = Self::create_graphics_pipeline(&self.device, self.swap_chain.dimensions(),
            &self.render_pass);
        self.swap_chain_framebuffers = Self::create_framebuffers(&self.swap_chain_images, &self.render_pass);
        self.create_command_buffers();
    }
}

fn main() {
    let mut app = HelloTriangleApplication::initialize();
    app.main_loop();
}