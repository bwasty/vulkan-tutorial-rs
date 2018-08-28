extern crate vulkano;
extern crate vulkano_win;
extern crate winit;

use std::sync::Arc;

use winit::{EventsLoop, WindowBuilder, dpi::LogicalSize, Event, WindowEvent};

use vulkano::instance::{
    Instance,
    InstanceExtensions,
    ApplicationInfo,
    Version,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

#[allow(unused)]
struct HelloTriangleApplication {
    instance: Arc<Instance>,
    events_loop: EventsLoop,
}

impl HelloTriangleApplication {
    pub fn initialize() -> Self {
        let instance = Self::create_instance();
        let events_loop = Self::init_window();

        Self {
            instance,
            events_loop,
        }
    }

    fn init_window() -> EventsLoop {
        let events_loop = EventsLoop::new();
        let _window_builder = WindowBuilder::new()
            .with_title("Vulkan")
            .with_dimensions(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)));
            // .build(&self.events_loop.as_ref().unwrap());
        events_loop
    }

    fn create_instance() -> Arc<Instance> {
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
        Instance::new(Some(&app_info), &required_extensions, None)
            .expect("failed to create Vulkan instance")
    }

    #[allow(unused)]
    fn main_loop(&mut self) {
        loop {
            let mut done = false;
            self.events_loop.poll_events(|ev| {
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
    let mut _app = HelloTriangleApplication::initialize();
    // app.main_loop();
}
