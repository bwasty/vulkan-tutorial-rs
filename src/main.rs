#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;
extern crate winit;

use std::sync::Arc;
use std::collections::HashSet;

use winit::{ WindowBuilder, dpi::LogicalSize, Event, WindowEvent};
use vulkano_win::VkSurfaceBuild;

use vulkano::instance::{
    Instance,
    InstanceExtensions,
    ApplicationInfo,
    Version,
    layers_list,
    PhysicalDevice,
    Features
};
use vulkano::instance::debug::{DebugCallback, MessageTypes};
use vulkano::device::{Device, DeviceExtensions, Queue};
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

fn vertices() -> [Vertex; 3] {
    [
        Vertex::new([0.0, -0.5], [1.0, 1.0, 1.0]),
        Vertex::new([0.5, 0.5], [0.0, 1.0, 0.0]),
        Vertex::new([-0.5, 0.5], [0.0, 0.0, 1.])
    ]
}

#[derive(Default)]
struct HelloTriangleApplication {
    instance: Option<Arc<Instance>>,
    debug_callback: Option<DebugCallback>,
    surface: Option<Arc<Surface<winit::Window>>>,

    physical_device_index: usize, // can't store PhysicalDevice directly (lifetime issues)
    device: Option<Arc<Device>>,

    graphics_queue: Option<Arc<Queue>>,
    present_queue: Option<Arc<Queue>>,

    swap_chain: Option<Arc<Swapchain<winit::Window>>>,
    swap_chain_images: Option<Vec<Arc<SwapchainImage<winit::Window>>>>,
    swap_chain_image_format: Option<Format>,
    swap_chain_extent: Option<[u32; 2]>,

    render_pass: Option<Arc<RenderPassAbstract + Send + Sync>>,
    graphics_pipeline: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,

    swap_chain_framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,

    vertex_buffer: Option<Arc<BufferAccess + Send + Sync>>,
    command_buffers: Vec<Arc<AutoCommandBuffer>>,

    previous_frame_end: Option<Box<GpuFuture>>,
    recreate_swap_chain: bool,

    events_loop: Option<winit::EventsLoop>,
}

impl HelloTriangleApplication {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(&mut self) {
        self.init_vulkan();
        self.main_loop();
    }

    fn init_vulkan(&mut self) {
        self.create_instance();
        self.setup_debug_callback();
        self.create_surface();
        self.pick_physical_device();
        self.create_logical_device();
        self.create_swap_chain();
        self.create_render_pass();
        self.create_graphics_pipeline();
        self.create_framebuffers();
        self.create_vertex_buffer();
        self.create_command_buffers();
        self.create_sync_objects();
    }

    fn create_instance(&mut self) {
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

        let instance =
            if ENABLE_VALIDATION_LAYERS && Self::check_validation_layer_support() {
                Instance::new(Some(&app_info), &required_extensions, VALIDATION_LAYERS.iter().map(|s| *s))
                    .expect("failed to create Vulkan instance")
            } else {
                Instance::new(Some(&app_info), &required_extensions, None)
                    .expect("failed to create Vulkan instance")
            };
        self.instance = Some(instance);
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

    fn setup_debug_callback(&mut self) {
        if !ENABLE_VALIDATION_LAYERS  {
            return;
        }

        let instance = self.instance.as_ref().unwrap();
        let msg_types = MessageTypes {
            error: true,
            warning: true,
            performance_warning: true,
            information: false,
            debug: true,
        };
        self.debug_callback = DebugCallback::new(instance, msg_types, |msg| {
            println!("validation layer: {:?}", msg.description);
        }).ok();
    }

    fn pick_physical_device(&mut self) {
        self.physical_device_index = PhysicalDevice::enumerate(&self.instance())
            .position(|device| self.is_device_suitable(&device))
            .expect("failed to find a suitable GPU!");
    }

    fn is_device_suitable(&self, device: &PhysicalDevice) -> bool {
        let indices = self.find_queue_families(device);
        let extensions_supported = Self::check_device_extension_support(device);

        let swap_chain_adequate = if extensions_supported {
                let capabilities = self.query_swap_chain_support(device);
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

    fn query_swap_chain_support(&self, device: &PhysicalDevice) -> Capabilities {
        self.surface.as_ref().unwrap().capabilities(*device)
            .expect("failed to get surface capabilities")
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

    fn choose_swap_extent(&self, capabilities: &Capabilities) -> [u32; 2] {
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

    fn create_swap_chain(&mut self) {
        let instance = self.instance.as_ref().unwrap();
        let physical_device = PhysicalDevice::from_index(instance, self.physical_device_index).unwrap();

        let capabilities = self.query_swap_chain_support(&physical_device);

        let surface_format = Self::choose_swap_surface_format(&capabilities.supported_formats);
        let present_mode = Self::choose_swap_present_mode(capabilities.present_modes);
        let extent = self.choose_swap_extent(&capabilities);

        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count.is_some() && image_count > capabilities.max_image_count.unwrap() {
            image_count = capabilities.max_image_count.unwrap();
        }

        let image_usage = ImageUsage {
            color_attachment: true,
            .. ImageUsage::none()
        };

        let indices = self.find_queue_families(&physical_device);

        let sharing: SharingMode = if indices.graphics_family != indices.present_family {
            vec![self.graphics_queue.as_ref().unwrap(), self.present_queue.as_ref().unwrap()].as_slice().into()
        } else {
            self.graphics_queue.as_ref().unwrap().into()
        };

        let (swap_chain, images) = Swapchain::new(
            self.device.as_ref().unwrap().clone(),
            self.surface.as_ref().unwrap().clone(),
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
            self.swap_chain.as_ref(), // old_swapchain
        ).expect("failed to create swap chain!");

        self.swap_chain = Some(swap_chain);
        self.swap_chain_images = Some(images);
        self.swap_chain_image_format = Some(surface_format.0);
        self.swap_chain_extent = Some(extent);
    }

    fn create_render_pass(&mut self) {
        self.render_pass = Some(Arc::new(single_pass_renderpass!(self.device().clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: self.swap_chain.as_ref().unwrap().format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap()));
    }

    fn create_graphics_pipeline(&mut self) {
        #[allow(unused)]
        mod vertex_shader {
            #[derive(VulkanoShader)]
            #[ty = "vertex"]
            #[path = "src/shaders/shader.vert"]
            struct Dummy;
        }

        #[allow(unused)]
        mod fragment_shader {
            #[derive(VulkanoShader)]
            #[ty = "fragment"]
            #[path = "src/shaders/shader.frag"]
            struct Dummy;
        }

        let device = self.device.as_ref().unwrap();
        let vert_shader_module = vertex_shader::Shader::load(device.clone())
            .expect("failed to create vertex shader module!");
        let frag_shader_module = fragment_shader::Shader::load(device.clone())
            .expect("failed to create fragment shader module!");

        let swap_chain_extent = self.swap_chain_extent.unwrap();
        let dimensions = [swap_chain_extent[0] as f32, swap_chain_extent[1] as f32];
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions,
            depth_range: 0.0 .. 1.0,
        };

        self.graphics_pipeline = Some(Arc::new(GraphicsPipeline::start()
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
            .front_face_clockwise()
            // NOTE: no depth_bias here, but on pipeline::raster::Rasterization
            .blend_pass_through() // = default
            .render_pass(Subpass::from(self.render_pass.as_ref().unwrap().clone(), 0).unwrap())
            .build(device.clone())
            .unwrap()
        ));
    }

    fn create_framebuffers(&mut self) {
        self.swap_chain_framebuffers = self.swap_chain_images.as_ref().unwrap().iter()
            .map(|image| {
                let fba: Arc<FramebufferAbstract + Send + Sync> = Arc::new(Framebuffer::start(self.render_pass.as_ref().unwrap().clone())
                    .add(image.clone()).unwrap()
                    .build().unwrap());
                fba
            }
        ).collect::<Vec<_>>();
    }

    fn create_vertex_buffer(&mut self) {
        let (buffer, future) = ImmutableBuffer::from_iter(
            vertices().iter().cloned(), BufferUsage::vertex_buffer(),
            self.graphics_queue().clone())
            .unwrap();
        future.flush().unwrap();
        self.vertex_buffer = Some(buffer);
    }

    fn create_command_buffers(&mut self) {
        let queue_family = self.graphics_queue.as_ref().unwrap().family();
        let graphics_pipeline = self.graphics_pipeline.as_ref().unwrap();
        self.command_buffers = self.swap_chain_framebuffers.iter()
            .map(|framebuffer| {
                Arc::new(AutoCommandBufferBuilder::primary_simultaneous_use(self.device().clone(), queue_family)
                    .unwrap()
                    .begin_render_pass(framebuffer.clone(), false, vec![[0.0, 0.0, 0.0, 1.0].into()])
                    .unwrap()
                    .draw(graphics_pipeline.clone(), &DynamicState::none(),
                        vec![self.vertex_buffer.as_ref().unwrap().clone()], (), ())
                    .unwrap()
                    .end_render_pass()
                    .unwrap()
                    .build()
                    .unwrap())
            })
            .collect();
    }

    fn create_sync_objects(&mut self) {
        self.previous_frame_end =
            Some(Box::new(sync::now(self.device().clone())) as Box<GpuFuture>);
    }

    fn find_queue_families(&self, device: &PhysicalDevice) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices::new();
        // TODO: replace index with id to simplify?
        for (i, queue_family) in device.queue_families().enumerate() {
            if queue_family.supports_graphics() {
                indices.graphics_family = i as i32;
            }

            if self.surface.as_ref().unwrap().is_supported(queue_family).unwrap() {
                indices.present_family = i as i32;
            }

            if indices.is_complete() {
                break;
            }
        }

        indices
    }

    fn create_logical_device(&mut self) {
        let instance = self.instance.as_ref().unwrap();
        let physical_device = PhysicalDevice::from_index(instance, self.physical_device_index).unwrap();

        let indices = self.find_queue_families(&physical_device);

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

        self.device = Some(device);

        // TODO!: simplify
        self.graphics_queue = queues
            .find(|q| q.family().id() == physical_device.queue_families().nth(indices.graphics_family as usize).unwrap().id());
        self.present_queue = queues
            .find(|q| q.family().id() == physical_device.queue_families().nth(indices.present_family as usize).unwrap().id());
    }

    fn create_surface(&mut self) {
        self.events_loop = Some(winit::EventsLoop::new());
        self.surface = WindowBuilder::new()
            .with_title("Vulkan")
            .with_dimensions(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
            .build_vk_surface(&self.events_loop.as_ref().unwrap(), self.instance().clone())
            .expect("failed to create window surface!")
            .into();
    }

    #[allow(unused)]
    fn main_loop(&mut self) {
        loop {
            self.draw_frame();

            let mut done = false;
            self.events_loop.as_mut().unwrap().poll_events(|ev| {
                match ev {
                    Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => done = true,
                    _ => ()
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

        let swap_chain = self.swap_chain().clone();
        let (image_index, acquire_future) = match acquire_next_image(swap_chain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                self.recreate_swap_chain = true;
                return;
            },
            Err(err) => panic!("{:?}", err)
        };

        let queue = self.graphics_queue().clone();
        let command_buffer = self.command_buffers[image_index].clone();

        let future = self.previous_frame_end.take().unwrap()
            .join(acquire_future)
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(queue.clone(), swap_chain.clone(), image_index)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(Box::new(future) as Box<_>);
            }
            Err(vulkano::sync::FlushError::OutOfDate) => {
                self.recreate_swap_chain = true;
                self.previous_frame_end
                    = Some(Box::new(vulkano::sync::now(self.device().clone())) as Box<_>);
            }
            Err(e) => {
                println!("{:?}", e);
                self.previous_frame_end
                    = Some(Box::new(vulkano::sync::now(self.device().clone())) as Box<_>);
            }
        }
    }

    fn recreate_swap_chain(&mut self) {
        unsafe { self.device().wait().unwrap(); }

        self.create_swap_chain();
        self.create_render_pass();
        self.create_graphics_pipeline();
        self.create_framebuffers();
        self.create_command_buffers();
    }

    fn instance(&self) -> &Arc<Instance> {
        self.instance.as_ref().unwrap()
    }

    fn device(&self) -> &Arc<Device> {
        self.device.as_ref().unwrap()
    }

    fn graphics_queue(&self) -> &Arc<Queue> {
        self.graphics_queue.as_ref().unwrap()
    }

    fn swap_chain(&self) -> &Arc<Swapchain<winit::Window>> {
        self.swap_chain.as_ref().unwrap()
    }
}

fn main() {
    let mut app = HelloTriangleApplication::new();
    app.run();
}
