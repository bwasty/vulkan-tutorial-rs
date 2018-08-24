extern crate vulkano;
extern crate vulkano_win;
extern crate winit;

use std::sync::Arc;

use winit::{ WindowBuilder, dpi::LogicalSize, Event, WindowEvent};
use vulkano::instance::{
    Instance,
    InstanceExtensions,
    ApplicationInfo,
    Version,
    layers_list,
};
use vulkano::instance::debug::{DebugCallback, MessageTypes};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const VALIDATION_LAYERS: &[&str] =  &[
    "VK_LAYER_LUNARG_standard_validation"
];

#[cfg(all(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

#[derive(Default)]
struct HelloTriangleApplication {
    instance: Option<Arc<Instance>>,
    debug_callback: Option<DebugCallback>,

    events_loop: Option<winit::EventsLoop>,
}

impl HelloTriangleApplication {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(&mut self) {
        self.init_window();
        self.init_vulkan();
        // self.main_loop();
    }

    fn init_window(&mut self) {
        self.events_loop = Some(winit::EventsLoop::new());
        // We'll leave this and the main loop commented out until we actually
        // have something to show on screen.
        let _window_builder = WindowBuilder::new()
            .with_title("Vulkan")
            .with_dimensions(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)));
            // .build(&self.events_loop.as_ref().unwrap());
    }

    fn init_vulkan(&mut self) {
        self.create_instance();
        self.setup_debug_callback();
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
}

fn main() {
    let mut app = HelloTriangleApplication::new();
    app.run();
}
