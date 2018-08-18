extern crate vulkano;
extern crate winit;
extern crate vulkano_win;

use std::sync::Arc;

use vulkano::instance::{ Instance, ApplicationInfo, Version, InstanceExtensions };

use winit::WindowBuilder;
use winit::dpi::LogicalSize;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

struct HelloTriangleApplication {
    instance: Option<Arc<Instance>>
}
impl HelloTriangleApplication {
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
    }

    fn create_instance(&mut self) {
        let extensions = InstanceExtensions::supported_by_core()
            .expect("failed to retrieve supported extensions");
        println!("Supported extensions: {:?}", extensions);

        let app_info = ApplicationInfo {
            application_name: Some("Hello Triangle".into()),
            application_version: Some(Version { major: 1, minor: 0, patch: 0 }),
            engine_name: Some("No Engine".into()),
            engine_version: Some(Version { major: 1, minor: 0, patch: 0 }),
        };

        let extensions = vulkano_win::required_extensions();
        self.instance = Some(Instance::new(Some(&app_info), &extensions, None)
            .expect("failed to create Vulkan instance"))
    }

    fn main_loop(&self) {

    }

    fn cleanup(&self) {

    }
}

fn main() {
    let mut app = HelloTriangleApplication { instance: None };
    app.run();
}
