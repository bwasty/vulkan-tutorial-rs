extern crate vulkano;
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
};
use vulkano::format::Format;
use vulkano::image::{ImageUsage, swapchain::SwapchainImage};
use vulkano::sync::SharingMode;

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

    events_loop: Option<winit::EventsLoop>,
}

impl HelloTriangleApplication {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(&mut self) {
        self.init_vulkan();
        // self.main_loop();
    }

    fn init_vulkan(&mut self) {
        self.create_instance();
        self.setup_debug_callback();
        self.create_surface();
        self.pick_physical_device();
        self.create_logical_device();
        self.create_swap_chain();
        self.create_graphics_pipeline();
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
            None, // old_swapchain
        ).expect("failed to create swap chain!");

        self.swap_chain = Some(swap_chain);
        self.swap_chain_images = Some(images);
        self.swap_chain_image_format = Some(surface_format.0);
        self.swap_chain_extent = Some(extent);
    }

    fn create_graphics_pipeline(&mut self) {

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

    fn instance(&self) -> &Arc<Instance> {
        self.instance.as_ref().unwrap()
    }
}

fn main() {
    let mut app = HelloTriangleApplication::new();
    app.run();
}
