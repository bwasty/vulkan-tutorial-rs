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
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

#[derive(Default)]
struct HelloTriangleApplication {
    instance: Option<Arc<Instance>>,

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
    }

    fn create_instance(&mut self) {
        let supported_extensions = InstanceExtensions::supported_by_core()
            .expect("failed to retrieve supported extensions");
        println!("Supported extensions: {:?}", supported_extensions);

        let app_info = ApplicationInfo {
            application_name: Some("Hello Triangle".into()),
            application_version: Some(Version { major: 1, minor: 0, patch: 0 }),
            engine_name: Some("No Engine".into()),
            engine_version: Some(Version { major: 1, minor: 0, patch: 0 }),
        };

        let required_extensions = vulkano_win::required_extensions();
        self.instance = Some(Instance::new(Some(&app_info), &required_extensions, None)
            .expect("failed to create Vulkan instance"))
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
