extern crate vulkano;
extern crate winit;
extern crate vulkano_win;

use std::sync::Arc;
use std::collections::HashSet;

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
use vulkano::swapchain::Surface;

use winit::WindowBuilder;
use winit::dpi::LogicalSize;
use vulkano_win::VkSurfaceBuild;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const VALIDATION_LAYERS: &'static [&'static str] =  &[
    "VK_LAYER_LUNARG_standard_validation"
];

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

    physical_device_index: usize, // can't store PhysicalDevice directly (lifetime issues)
    device: Option<Arc<Device>>,
    graphics_queue: Option<Arc<Queue>>,
    present_queue: Option<Arc<Queue>>,
    surface: Option<Arc<Surface<winit::Window>>>,
}
#[allow(dead_code)] // TODO: TMP
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
            .with_dimensions(LogicalSize::new(WIDTH as f64, HEIGHT as f64));
    }

    fn init_vulkan(&mut self) {
        self.create_instance();
        self.setup_debug_callback();
        self.create_surface();
        self.pick_physical_device();
        self.create_logical_device();
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
        let instance = self.instance.as_ref().unwrap();
        self.physical_device_index = PhysicalDevice::enumerate(&instance)
            .position(|device| Self::is_device_suitable(&device))
            .expect("failed to find a suitable GPU!");
    }

    fn is_device_suitable(device: &PhysicalDevice) -> bool {
        Self::find_queue_families(device).is_complete()
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
            &DeviceExtensions::none(), queue_families)
            .expect("failed to create logical device!");

        self.device = Some(device);

        // TODO!: simplify
        self.graphics_queue = queues
            .find(|q| q.family().id() == physical_device.queue_families().nth(indices.graphics_family as usize).unwrap().id());
        self.present_queue = queues
            .find(|q| q.family().id() == physical_device.queue_families().nth(indices.present_family as usize).unwrap().id());
    }

    fn create_surface(&mut self) {
        let instance = self.instance.as_ref().unwrap();

        let /*mut*/ events_loop = winit::EventsLoop::new();
        self.surface = WindowBuilder::new().build_vk_surface(&events_loop, instance.clone())
            .expect("failed to create window surface!")
            .into();
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

        return true;
    }

    fn get_required_extensions() -> InstanceExtensions {
        let mut extensions = vulkano_win::required_extensions();
        if ENABLE_VALIDATION_LAYERS {
            // TODO!: this should be ext_debug_utils (_report is deprecated), but that doesn't exist yet in vulkano
            extensions.ext_debug_report = true;
        }

        return extensions;
    }

    fn main_loop(&self) {

    }

    fn cleanup(&self) {
        // TODO!: trust automatic drop and remove or use std::mem::drop here? (instance, device etc.)
        // -> check with validation layers for issues with order...
    }
}

fn main() {
    let mut app = HelloTriangleApplication::new();
    app.run();
}
