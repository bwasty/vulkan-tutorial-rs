# vulkan-tutorial-rs
Rust version of https://github.com/Overv/VulkanTutorial.

**Goal**: Rust port with code structure as similar as possible to the original C++, so the original tutorial can easily be followed (similar to [learn-opengl-rs](https://github.com/bwasty/learn-opengl-rs)).

**Current State**: Early, got the code up to [Swap chain recreation](https://vulkan-tutorial.com/Drawing_a_triangle/Swap_chain_recreation) (the triangle renders!), but it isn't yet (fully) split up by chapter and the notes below are incomplete.

---
* [Introduction](#introduction)
* [Overview](#overview)
* [Development Environment](#development-environment)
* [Drawing a triangle](#drawing-a-triangle)
    * [Setup](#setup)
    * [Base code](#base-code)
        * [General structure](#general-structure)
        * [Resource management](#resource-management)
        * [Integrating <del>GLFW</del> winit](#integrating-glfw-winit)
    * [Instance](#instance)
    * [Validation layers](#validation-layers)
    * [Physical devices and queue families](#physical-devices-and-queue-families)
    * [Logical device and queues](#logical-device-and-queues)
    * [Presentation](#presentation)
    * [Graphics pipeline basics](#graphics-pipeline-basics)
    * [Drawing](#drawing)
    * [Swapchain recreation](#swapchain-recreation)

## Introduction
This tutorial consists of the the ported code and notes about the differences between the original C++ and the Rust code.
The [explanatory texts](https://vulkan-tutorial.com/Introduction) generally apply equally, although the Rust version is often shorter due to the use of [Vulkano](http://vulkano.rs/), a safe wrapper around the Vulkan API.

## Overview
https://vulkan-tutorial.com/Overview

(nothing to note here)

## Development Environment
https://vulkan-tutorial.com/Development_environment

Download the Vulkan SDK as described, but ignore everything about library and project setup. Instead, create a new Cargo project:
```
$ cargo new vulkan-tutorial-rs
```
Then add this to your `Cargo.toml`:
```
[dependencies]
vulkano = "0.10.0"
```

On macOS, copy [mac-env.sh](mac-env.sh), adapt the `VULKAN_SDK` if necessary and `source` the file in your terminal. See also [vulkano-rs/vulkano#macos-and-ios-setup](https://github.com/vulkano-rs/vulkano#macos-and-ios-setup).

## Drawing a triangle
### Setup
#### Base code
https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Base_code
##### General structure
```rust
extern crate vulkano;

#[derive(Default)]
struct HelloTriangleApplication {

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

    }

    fn main_loop(&mut self) {

    }
}

fn main() {
    let mut app = HelloTriangleApplication::new();
    app.run();
}
```

##### Resource management
Vulkano handles calling `vkDestroyXXX`/`vkFreeXXX` in the `Drop` implementation of all wrapper objects, so we will skip all cleanup code.

##### Integrating ~GLFW~ winit
Instead of GLFW we're using [winit](https://github.com/tomaka/winit), an alternative window managment library written in pure Rust.

Add this to your Cargo.toml:
```
winit = "0.17.1"
```
And extend your main.rs:
```rust
extern crate winit;

use winit::{ WindowBuilder, dpi::LogicalSize};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
```
```rust
    pub fn run(&mut self) {
        self.init_window();
        self.init_vulkan();
        self.main_loop();
    }

    fn init_window(&mut self) {
        self.events_loop = Some(winit::EventsLoop::new());
        let _window = WindowBuilder::new()
            .with_title("Vulkan")
            .with_dimensions(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
            .build(&self.events_loop.as_ref().unwrap());
    }
```
```rust
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
```

[Complete code](src/bin/00_base_code.rs)


#### Instance
https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Instance

```rust
extern crate vulkano_win;
```
```rust
use vulkano::instance::{
    Instance,
    InstanceExtensions,
    ApplicationInfo,
    Version,
};
```
```rust
struct HelloTriangleApplication {
    instance: Option<Arc<Instance>>,
    ...
}
```
```rust
    fn init_vulkan(&mut self) {
        self.create_instance();
    }
```
```rust
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
```

[Complete code](src/bin/01_instance_creation.rs)

#### Validation layers
https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Validation_layers
```diff
--- a/01_instance_creation.rs
+++ b/02_validation_layers.rs
@@ -10,14 +10,26 @@ use vulkano::instance::{
     InstanceExtensions,
     ApplicationInfo,
     Version,
+    layers_list,
 };
+use vulkano::instance::debug::{DebugCallback, MessageTypes};

 const WIDTH: u32 = 800;
 const HEIGHT: u32 = 600;

+const VALIDATION_LAYERS: &[&str] =  &[
+    "VK_LAYER_LUNARG_standard_validation"
+];
+
+#[cfg(all(debug_assertions))]
+const ENABLE_VALIDATION_LAYERS: bool = true;
+#[cfg(not(debug_assertions))]
+const ENABLE_VALIDATION_LAYERS: bool = false;
+
 #[derive(Default)]
 struct HelloTriangleApplication {
     instance: Option<Arc<Instance>>,
+    debug_callback: Option<DebugCallback>,

     events_loop: Option<winit::EventsLoop>,
 }
@@ -45,9 +57,14 @@ impl HelloTriangleApplication {

     fn init_vulkan(&mut self) {
         self.create_instance();
+        self.setup_debug_callback();
     }

     fn create_instance(&mut self) {
+        if ENABLE_VALIDATION_LAYERS && !Self::check_validation_layer_support() {
+            println!("Validation layers requested, but not available!")
+        }
+
         let supported_extensions = InstanceExtensions::supported_by_core()
             .expect("failed to retrieve supported extensions");
         println!("Supported extensions: {:?}", supported_extensions);
@@ -59,9 +76,51 @@ impl HelloTriangleApplication {
             engine_version: Some(Version { major: 1, minor: 0, patch: 0 }),
         };

-        let required_extensions = vulkano_win::required_extensions();
-        self.instance = Some(Instance::new(Some(&app_info), &required_extensions, None)
-            .expect("failed to create Vulkan instance"))
+        let required_extensions = Self::get_required_extensions();
+
+        let instance =
+            if ENABLE_VALIDATION_LAYERS && Self::check_validation_layer_support() {
+                Instance::new(Some(&app_info), &required_extensions, VALIDATION_LAYERS.iter().map(|s| *s))
+                    .expect("failed to create Vulkan instance")
+            } else {
+                Instance::new(Some(&app_info), &required_extensions, None)
+                    .expect("failed to create Vulkan instance")
+            };
+        self.instance = Some(instance);
+    }
+
+    fn check_validation_layer_support() -> bool {
+        let layers: Vec<_> = layers_list().unwrap().map(|l| l.name().to_owned()).collect();
+        VALIDATION_LAYERS.iter()
+            .all(|layer_name| layers.contains(&layer_name.to_string()))
+    }
+
+    fn get_required_extensions() -> InstanceExtensions {
+        let mut extensions = vulkano_win::required_extensions();
+        if ENABLE_VALIDATION_LAYERS {
+            // TODO!: this should be ext_debug_utils (_report is deprecated), but that doesn't exist yet in vulkano
+            extensions.ext_debug_report = true;
+        }
+
+        extensions
+    }
+
+    fn setup_debug_callback(&mut self) {
+        if !ENABLE_VALIDATION_LAYERS  {
+            return;
+        }
+
+        let instance = self.instance.as_ref().unwrap();
+        let msg_types = MessageTypes {
+            error: true,
+            warning: true,
+            performance_warning: true,
+            information: false,
+            debug: true,
+        };
+        self.debug_callback = DebugCallback::new(instance, msg_types, |msg| {
+            println!("validation layer: {:?}", msg.description);
+        }).ok();
     }
```

[Complete code](src/bin/02_validation_layers.rs)


#### Physical devices and queue families
https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Physical_devices_and_queue_families

```diff
--- a/02_validation_layers.rs
+++ b/03_physical_device_selection.rs
@@ -11,6 +11,7 @@ use vulkano::instance::{
     ApplicationInfo,
     Version,
     layers_list,
+    PhysicalDevice,
 };
 use vulkano::instance::debug::{DebugCallback, MessageTypes};

@@ -26,10 +27,25 @@ const ENABLE_VALIDATION_LAYERS: bool = true;
 #[cfg(not(debug_assertions))]
 const ENABLE_VALIDATION_LAYERS: bool = false;

+struct QueueFamilyIndices {
+    graphics_family: i32,
+    present_family: i32,
+}
+impl QueueFamilyIndices {
+    fn new() -> Self {
+        Self { graphics_family: -1, present_family: -1 }
+    }
+
+    fn is_complete(&self) -> bool {
+        self.graphics_family >= 0 && self.present_family >= 0
+    }
+}
+
 #[derive(Default)]
 struct HelloTriangleApplication {
     instance: Option<Arc<Instance>>,
     debug_callback: Option<DebugCallback>,
+    physical_device_index: usize, // can't store PhysicalDevice directly (lifetime issues)

     events_loop: Option<winit::EventsLoop>,
 }
@@ -58,6 +74,7 @@ impl HelloTriangleApplication {
     fn init_vulkan(&mut self) {
         self.create_instance();
         self.setup_debug_callback();
+        self.pick_physical_device();
     }

     fn create_instance(&mut self) {
@@ -123,6 +140,33 @@ impl HelloTriangleApplication {
         }).ok();
     }

+    fn pick_physical_device(&mut self) {
+        self.physical_device_index = PhysicalDevice::enumerate(&self.instance())
+            .position(|device| self.is_device_suitable(&device))
+            .expect("failed to find a suitable GPU!");
+    }
+
+    fn is_device_suitable(&self, device: &PhysicalDevice) -> bool {
+        let indices = self.find_queue_families(device);
+        indices.is_complete()
+    }
+
+    fn find_queue_families(&self, device: &PhysicalDevice) -> QueueFamilyIndices {
+        let mut indices = QueueFamilyIndices::new();
+        // TODO: replace index with id to simplify?
+        for (i, queue_family) in device.queue_families().enumerate() {
+            if queue_family.supports_graphics() {
+                indices.graphics_family = i as i32;
+            }
+
+            if indices.is_complete() {
+                break;
+            }
+        }
+
+        indices
+    }
+
     #[allow(unused)]
     fn main_loop(&mut self) {
         loop {
@@ -138,6 +182,10 @@ impl HelloTriangleApplication {
             }
         }
     }
+
+    fn instance(&self) -> &Arc<Instance> {
+        self.instance.as_ref().unwrap()
+    }
 }

 fn main() {
```
[Complete code](src/bin/03_physical_device_selection.rs)


#### Logical device and queues
https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Logical_device_and_queues

*TODO*

### Presentation

*TODO*

### Graphics pipeline basics

*TODO*

### Drawing

*TODO*

### Swapchain recreation

*TODO*

[Complete code](src/main.rs)
