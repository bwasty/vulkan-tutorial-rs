#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate winit;
extern crate vulkano_win;

use std::sync::Arc;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;

use vulkano::instance::{
    Instance,
    InstanceExtensions,
    layers_list,
    ApplicationInfo,
    Version,
    PhysicalDevice,
    Features,
};
use vulkano::instance::debug::{DebugCallback, MessageTypes};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::swapchain::{
    Surface,
    Capabilities,
    ColorSpace,
    SupportedPresentModes, PresentMode,
    Swapchain,
    CompositeAlpha,
};
use vulkano::format::{Format};
use vulkano::image::{
    ImageUsage,
    swapchain::SwapchainImage
};
use vulkano::sync::SharingMode;
use vulkano::command_buffer::{
    AutoCommandBuffer,
    AutoCommandBufferBuilder,
    DynamicState,
};
use vulkano::pipeline::{
    shader::ShaderModule,
    GraphicsPipeline,
    GraphicsPipelineAbstract,
    vertex::BufferlessDefinition,
    vertex::BufferlessVertices,
    viewport::Viewport,
};
use vulkano::framebuffer::{
    Subpass,
    RenderPassAbstract,
    Framebuffer,
    FramebufferAbstract,
};

use winit::WindowBuilder;
use winit::dpi::LogicalSize;
use vulkano_win::VkSurfaceBuild;

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

// MoltenVK doesn't have any layers by default
#[cfg(all(debug_assertions, not(target_os = "macos")))]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(any(not(debug_assertions), target_os = "macos"))]
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

    // command_pool: Option<Arc<StandardCommandPool>>,
    command_buffers: Vec<AutoCommandBuffer>,
}
impl HelloTriangleApplication {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(&mut self) {
        self.init_window();
        self.init_vulkan();
        self.main_loop();
        self.cleanup();
    }

    fn init_window(&self) {
        WindowBuilder::new()
            .with_title("Vulkan")
            .with_dimensions(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)));
    }

    fn init_vulkan(&mut self) {
        self.create_instance();
        self.setup_debug_callback();
        self.create_surface();
        self.pick_physical_device();
        self.create_logical_device();
        self.create_swap_chain();
        // NOTE: no `create_image_views`  becayse image views are handled by
        // Vulkano and can be accessed via the SwapchainImages created above
        self.create_render_pass();
        // See create_command_buffers
        // self.create_graphics_pipeline();
        self.create_framebuffers();
        // NOTE: Vulkano has a `StandardCommandPool` that is used automatically,
        // but it is possible to use custom pools. See the vulkano::command_buffer
        // module docs for details
        // self.create_command_pool();
        self.create_command_buffers();
    }

    fn create_instance(&mut self) {
        if ENABLE_VALIDATION_LAYERS && !Self::check_validation_layer_support() {
            panic!("validation layers requested, but not available!")
        }

        let extensions = InstanceExtensions::supported_by_core()
            .expect("failed to retrieve supported extensions");
        println!("Supported extensions: {:?}", extensions);

        let app_info = ApplicationInfo {
            application_name: Some("Hello Triangle".into()),
            application_version: Some(Version { major: 1, minor: 0, patch: 0 }),
            engine_name: Some("No Engine".into()),
            engine_version: Some(Version { major: 1, minor: 0, patch: 0 }),
        };

        let extensions = Self::get_required_extensions();

        let instance =
            if ENABLE_VALIDATION_LAYERS {
                Instance::new(Some(&app_info), &extensions, VALIDATION_LAYERS.iter().map(|s| *s))
                    .expect("failed to create Vulkan instance")
            } else {
                Instance::new(Some(&app_info), &extensions, None)
                    .expect("failed to create Vulkan instance")
            };
        self.instance = Some(instance);
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
        let indices = Self::find_queue_families(device);
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

    fn create_logical_device(&mut self) {
        let instance = self.instance.as_ref().unwrap();
        let physical_device = PhysicalDevice::from_index(instance, self.physical_device_index).unwrap();

        let indices = Self::find_queue_families(&physical_device);

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
        let /*mut*/ events_loop = winit::EventsLoop::new();
        self.surface = WindowBuilder::new().build_vk_surface(&events_loop, self.instance().clone())
            .expect("failed to create window surface!")
            .into();
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

    fn create_swap_chain(&mut self) {
        let instance = self.instance.as_ref().unwrap();
        let physical_device = PhysicalDevice::from_index(instance, self.physical_device_index).unwrap();

        let capabilities = self.query_swap_chain_support(&physical_device);

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

        let indices = Self::find_queue_families(&physical_device);

        let sharing: SharingMode = if indices.graphics_family != indices.present_family {
            vec![self.graphics_queue.as_ref().unwrap(), self.present_queue.as_ref().unwrap()].as_slice().into()
        } else {
            self.graphics_queue.as_ref().unwrap().into()
        };

        let (swap_chain, images) = Swapchain::new(
            self.device.as_ref().unwrap().clone(),
            self.surface.as_ref().unwrap().clone(),
            image_count,
            surface_format.0, // TODO!? (color space?)
            extent,
            1, // layers
            image_usage,
            sharing,
            capabilities.current_transform,
            CompositeAlpha::Opaque,
            present_mode,
            true, // clipped
            None, // old_swapchain
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

    #[allow(unused)]
    fn create_graphics_pipeline(&mut self) {
        // NOTE: the standard vulkano way is to load shaders as GLSL at
        // compile-time via macros from the vulkano_shader_derive crate.
        // Loading SPIR-V at runtime like in the C++ version is partially
        // implemented, but currently unused.

        // let vert_shader_code = Self::read_file("src/shaders/vert.spv");
        // let frag_shader_code = Self::read_file("src/shaders/frag.spv");
        // let vert_shader_module = self.create_shader_module(&vert_shader_code);
        // let frag_shader_module = self.create_shader_module(&frag_shader_code);

        let device = self.device.as_ref().unwrap();
        let vert_shader_module = vertex_shader::Shader::load(device.clone())
            .expect("failed to create shader module!");
        let frag_shader_module = fragment_shader::Shader::load(device.clone())
            .expect("failed to create shader module!");

        self.graphics_pipeline = Some(Arc::new(GraphicsPipeline::start()
            .vertex_input(BufferlessDefinition {})
            .vertex_shader(vert_shader_module.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(frag_shader_module.main_entry_point(), ())
            .render_pass(Subpass::from(self.render_pass.as_ref().unwrap().clone(), 0).unwrap())
            .build(device.clone())
            .unwrap()));
        println!("GraphicsPipeline created!");
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
        println!("framebuffers created")
    }

    // TODO!!: remove because AutoCommandBufferBuilder uses it automatically?
    // fn create_command_pool(&mut self) {
    //     let device = self.device().clone();
    //     self.command_pool = Some(Device::standard_command_pool(&device,
    //         self.graphics_queue.as_ref().unwrap().family()));
    // }

    fn create_command_buffers(&mut self) {
        let swap_chain_extent = self.swap_chain_extent.unwrap();
        let dimensions = [swap_chain_extent[0] as f32, swap_chain_extent[1] as f32];
        let dynamic_state = DynamicState {
            viewports: Some(vec![Viewport {
                origin: [0.0, 0.0],
                dimensions,
                depth_range: 0.0 .. 1.0,
            }]),
            .. DynamicState::none()
        };

        // HACK:
        // We need to define the graphics_pipeline here instead of using
        // self.graphics_pipeline, because `BufferlessVertices` below only
        // works when the concrete type of the graphics pipeline is visible
        // to the command buffer.
        // Hopefully this can be removed when getting to the `Vertex Buffers` chapter
        let device = self.device.as_ref().unwrap();
        let vert_shader_module = vertex_shader::Shader::load(device.clone())
            .expect("failed to create shader module!");
        let frag_shader_module = fragment_shader::Shader::load(device.clone())
            .expect("failed to create shader module!");
        let graphics_pipeline = Arc::new(GraphicsPipeline::start()
            .vertex_input(BufferlessDefinition {})
            .vertex_shader(vert_shader_module.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(frag_shader_module.main_entry_point(), ())
            .render_pass(Subpass::from(self.render_pass.as_ref().unwrap().clone(), 0).unwrap())
            .build(device.clone())
            .unwrap());
        ////

        let queue_family = self.graphics_queue.as_ref().unwrap().family();
        // let graphics_pipeline = self.graphics_pipeline.as_ref().unwrap();
        self.command_buffers = self.swap_chain_framebuffers.iter()
            .map(|framebuffer| {
                let vertices = BufferlessVertices { vertices: 3, instances: 0 };
                AutoCommandBufferBuilder::primary_simultaneous_use(self.device().clone(), queue_family)
                    .unwrap()
                    .begin_render_pass(framebuffer.clone(), false, vec![[0.0, 0.0, 0.0, 1.0].into()])
                    .unwrap()
                    .draw(graphics_pipeline.clone(), &dynamic_state,
                        vertices, (), ())
                    .unwrap()
                    .end_render_pass()
                    .unwrap()
                    .build()
                    .unwrap()
            })
            .collect();
        println!("command buffers built.")
    }

    fn instance(&self) -> &Arc<Instance> {
        self.instance.as_ref().unwrap()
    }

    fn device(&self) -> &Arc<Device> {
        self.device.as_ref().unwrap()
    }

    // fn physical_device(&self) -> PhysicalDevice {
    //     PhysicalDevice::from_index(self.instance(), self.physical_device_index).unwrap()
    // }

    #[allow(unused)]
    fn read_file(filename: &str) -> Vec<u8> {
        let mut f = File::open(filename)
            .expect("failed to open file!");
        let mut buffer = vec![];
        f.read_to_end(&mut buffer).unwrap();
        buffer
    }

    #[allow(unused)]
    fn create_shader_module(&self, code: &[u8]) -> Arc<ShaderModule> {
        unsafe {
            ShaderModule::new(self.device.as_ref().unwrap().clone(), &code)
        }.expect("failed to create shader module!")
    }

    fn find_queue_families(device: &PhysicalDevice) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices::new();
        // TODO: replace index with id to simplify?
        for (i, queue_family) in device.queue_families().enumerate() {
            if queue_family.supports_graphics() {
                indices.graphics_family = i as i32;

                // TODO: Vulkano doesn't seem to support querying 'present support' (vkGetPhysicalDeviceSurfaceSupportKHR)
                // -> assuming it does if it supports graphics
                indices.present_family = i as i32;
            }


            if indices.is_complete() {
                break;
            }
        }

        indices
    }

    fn check_validation_layer_support() -> bool {
        // println!("Available layers:");
        // for layer in instance::layers_list().unwrap() {
        //     println!("{}", layer.name());
        // }
        for layer_name in VALIDATION_LAYERS.iter() {
            let mut layer_found = false;
            for layer_properties in layers_list().unwrap() {
                if *layer_name == layer_properties.name() {
                    layer_found = true;
                    break
                }
            }
            if !layer_found {
                return false;
            }
        }

        true
    }

    fn get_required_extensions() -> InstanceExtensions {
        let mut extensions = vulkano_win::required_extensions();
        if ENABLE_VALIDATION_LAYERS {
            // TODO!: this should be ext_debug_utils (_report is deprecated), but that doesn't exist yet in vulkano
            extensions.ext_debug_report = true;
        }

        extensions
    }

    fn main_loop(&self) {

    }

    fn cleanup(&self) {
        // TODO!: trust automatic drop and remove or use std::mem::drop here? (instance, device etc.)
        // -> check with validation layers for issues with order...
    }
}

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


fn main() {
    let mut app = HelloTriangleApplication::new();
    app.run();
}
