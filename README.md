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
      * [Window surface](#window-surface)
      * [Swap chain](#swap-chain)
      * [Image views](#image-views)
   * [Graphics pipeline basics](#graphics-pipeline-basics)
      * [Introduction](#introduction-1)
      * [Shader Modules](#shader-modules)
      * [Fixed functions](#fixed-functions)
      * [Render passes](#render-passes)
      * [Conclusion](#conclusion)
   * [Drawing (<em>TODO</em>)](#drawing-todo)
      * [Framebuffers](#framebuffers)
      * [Command buffers](#command-buffers)
      * [Rendering and presentation](#rendering-and-presentation)
   * [Swapchain recreation (<em>TODO</em>)](#swapchain-recreation-todo)
* [Vertex buffers (<em>TODO</em>)](#vertex-buffers-todo)
* [Uniform buffers (<em>TODO</em>)](#uniform-buffers-todo)
* [Texture mapping (<em>TODO</em>)](#texture-mapping-todo)
* [Depth buffering (<em>TODO</em>)](#depth-buffering-todo)
* [Loading models (<em>TODO</em>)](#loading-models-todo)
* [Generating Mipmaps (<em>TODO</em>)](#generating-mipmaps-todo)
* [Multisampling (<em>TODO</em>)](#multisampling-todo)

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

<details>
<summary>Diff</summary>

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
</details>

[Complete code](src/bin/02_validation_layers.rs)


#### Physical devices and queue families
https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Physical_devices_and_queue_families

<details>
<summary>Diff</summary>

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
</details>

[Complete code](src/bin/03_physical_device_selection.rs)


#### Logical device and queues
https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Logical_device_and_queues

<details>
<summary>Diff</summary>

```diff
--- a/03_physical_device_selection.rs
+++ b/04_logical_device.rs
@@ -12,8 +12,10 @@ use vulkano::instance::{
     Version,
     layers_list,
     PhysicalDevice,
+    Features
 };
 use vulkano::instance::debug::{DebugCallback, MessageTypes};
+use vulkano::device::{Device, DeviceExtensions, Queue};

 const WIDTH: u32 = 800;
 const HEIGHT: u32 = 600;
@@ -45,6 +47,9 @@ struct HelloTriangleApplication {
     instance: Option<Arc<Instance>>,
     debug_callback: Option<DebugCallback>,
     physical_device_index: usize, // can't store PhysicalDevice directly (lifetime issues)
+    device: Option<Arc<Device>>,
+
+    graphics_queue: Option<Arc<Queue>>,

     events_loop: Option<winit::EventsLoop>,
 }
@@ -74,6 +79,7 @@ impl HelloTriangleApplication {
         self.create_instance();
         self.setup_debug_callback();
         self.pick_physical_device();
+        self.create_logical_device();
     }

     fn create_instance(&mut self) {
@@ -166,6 +172,26 @@ impl HelloTriangleApplication {
         indices
     }

+    fn create_logical_device(&mut self) {
+        let instance = self.instance.as_ref().unwrap();
+        let physical_device = PhysicalDevice::from_index(instance, self.physical_device_index).unwrap();
+        let indices = self.find_queue_families(&physical_device);
+        let queue_family = physical_device.queue_families()
+            .nth(indices.graphics_family as usize).unwrap();
+        let queue_priority = 1.0;
+
+        // NOTE: the tutorial recommends passing the validation layers as well
+        // for legacy reasons (if ENABLE_VALIDATION_LAYERS is true). Vulkano handles that
+        // for us internally.
+
+        let (device, mut queues) = Device::new(physical_device, &Features::none(), &DeviceExtensions::none(),
+            [(queue_family, queue_priority)].iter().cloned())
+            .expect("failed to create logical device!");
+
+        self.device = Some(device);
+        self.graphics_queue = queues.next();
+    }
+
     #[allow(unused)]
     fn main_loop(&mut self) {
         loop {
```
</details>

[Complete code](src/bin/04_logical_device.rs)

### Presentation

#### Window surface
https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Window_surface
<details>
<summary>Diff</summary>

```diff
--- a/04_logical_device.rs
+++ b/05_window_surface.rs
@@ -3,8 +3,11 @@ extern crate vulkano_win;
 extern crate winit;

 use std::sync::Arc;
+use std::collections::HashSet;

 use winit::{ WindowBuilder, dpi::LogicalSize, Event, WindowEvent};
+use vulkano_win::VkSurfaceBuild;
+
 use vulkano::instance::{
     Instance,
     InstanceExtensions,
@@ -16,6 +19,9 @@ use vulkano::instance::{
 };
 use vulkano::instance::debug::{DebugCallback, MessageTypes};
 use vulkano::device::{Device, DeviceExtensions, Queue};
+use vulkano::swapchain::{
+    Surface,
+};

 const WIDTH: u32 = 800;
 const HEIGHT: u32 = 600;
@@ -31,14 +37,15 @@ const ENABLE_VALIDATION_LAYERS: bool = false;

 struct QueueFamilyIndices {
     graphics_family: i32,
+    present_family: i32,
 }
 impl QueueFamilyIndices {
     fn new() -> Self {
-        Self { graphics_family: -1 }
+        Self { graphics_family: -1, present_family: -1 }
     }

     fn is_complete(&self) -> bool {
-        self.graphics_family >= 0
+        self.graphics_family >= 0 && self.present_family >= 0
     }
 }

@@ -46,10 +53,13 @@ impl QueueFamilyIndices {
 struct HelloTriangleApplication {
     instance: Option<Arc<Instance>>,
     debug_callback: Option<DebugCallback>,
+    surface: Option<Arc<Surface<winit::Window>>>,
+
     physical_device_index: usize, // can't store PhysicalDevice directly (lifetime issues)
     device: Option<Arc<Device>>,

     graphics_queue: Option<Arc<Queue>>,
+    present_queue: Option<Arc<Queue>>,

     events_loop: Option<winit::EventsLoop>,
 }
@@ -60,24 +70,14 @@ impl HelloTriangleApplication {
     }

     pub fn run(&mut self) {
-        self.init_window();
         self.init_vulkan();
         // self.main_loop();
     }

-    fn init_window(&mut self) {
-        self.events_loop = Some(winit::EventsLoop::new());
-        // We'll leave this and the main loop commented out until we actually
-        // have something to show on screen.
-        let _window_builder = WindowBuilder::new()
-            .with_title("Vulkan")
-            .with_dimensions(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)));
-            // .build(&self.events_loop.as_ref().unwrap());
-    }
-
     fn init_vulkan(&mut self) {
         self.create_instance();
         self.setup_debug_callback();
+        self.create_surface();
         self.pick_physical_device();
         self.create_logical_device();
     }
@@ -164,6 +164,10 @@ impl HelloTriangleApplication {
                 indices.graphics_family = i as i32;
             }

+            if self.surface.as_ref().unwrap().is_supported(queue_family).unwrap() {
+                indices.present_family = i as i32;
+            }
+
             if indices.is_complete() {
                 break;
             }
@@ -175,21 +179,43 @@ impl HelloTriangleApplication {
     fn create_logical_device(&mut self) {
         let instance = self.instance.as_ref().unwrap();
         let physical_device = PhysicalDevice::from_index(instance, self.physical_device_index).unwrap();
+
         let indices = self.find_queue_families(&physical_device);
-        let queue_family = physical_device.queue_families()
-            .nth(indices.graphics_family as usize).unwrap();
+
+        let families = [indices.graphics_family, indices.present_family];
+        use std::iter::FromIterator;
+        let unique_queue_families: HashSet<&i32> = HashSet::from_iter(families.iter());
+
         let queue_priority = 1.0;
+        let queue_families = unique_queue_families.iter().map(|i| {
+            (physical_device.queue_families().nth(**i as usize).unwrap(), queue_priority)
+        });

         // NOTE: the tutorial recommends passing the validation layers as well
         // for legacy reasons (if ENABLE_VALIDATION_LAYERS is true). Vulkano handles that
         // for us internally.

-        let (device, mut queues) = Device::new(physical_device, &Features::none(), &DeviceExtensions::none(),
-            [(queue_family, queue_priority)].iter().cloned())
+        let (device, mut queues) = Device::new(physical_device, &Features::none(),
+            &DeviceExtensions::none(), queue_families)
             .expect("failed to create logical device!");

         self.device = Some(device);
-        self.graphics_queue = queues.next();
+
+        // TODO!: simplify
+        self.graphics_queue = queues
+            .find(|q| q.family().id() == physical_device.queue_families().nth(indices.graphics_family as usize).unwrap().id());
+        self.present_queue = queues
+            .find(|q| q.family().id() == physical_device.queue_families().nth(indices.present_family as usize).unwrap().id());
+    }
+
+    fn create_surface(&mut self) {
+        self.events_loop = Some(winit::EventsLoop::new());
+        self.surface = WindowBuilder::new()
+            .with_title("Vulkan")
+            .with_dimensions(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
+            .build_vk_surface(&self.events_loop.as_ref().unwrap(), self.instance().clone())
+            .expect("failed to create window surface!")
+            .into();
     }
```
</details>

[Complete code](src/bin/05_window_surface.rs)

#### Swap chain
https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Swap_chain
<details>
<summary>Diff</summary>

```diff
--- a/05_window_surface.rs
+++ b/06_swap_chain_creation.rs
@@ -21,7 +21,16 @@ use vulkano::instance::debug::{DebugCallback, MessageTypes};
 use vulkano::device::{Device, DeviceExtensions, Queue};
 use vulkano::swapchain::{
     Surface,
+    Capabilities,
+    ColorSpace,
+    SupportedPresentModes,
+    PresentMode,
+    Swapchain,
+    CompositeAlpha,
 };
+use vulkano::format::Format;
+use vulkano::image::{ImageUsage, swapchain::SwapchainImage};
+use vulkano::sync::SharingMode;

 const WIDTH: u32 = 800;
 const HEIGHT: u32 = 600;
@@ -30,6 +39,14 @@ const VALIDATION_LAYERS: &[&str] =  &[
     "VK_LAYER_LUNARG_standard_validation"
 ];

+/// Required device extensions
+fn device_extensions() -> DeviceExtensions {
+    DeviceExtensions {
+        khr_swapchain: true,
+        .. vulkano::device::DeviceExtensions::none()
+    }
+}
+
 #[cfg(all(debug_assertions))]
 const ENABLE_VALIDATION_LAYERS: bool = true;
 #[cfg(not(debug_assertions))]
@@ -61,6 +78,11 @@ struct HelloTriangleApplication {
     graphics_queue: Option<Arc<Queue>>,
     present_queue: Option<Arc<Queue>>,

+    swap_chain: Option<Arc<Swapchain<winit::Window>>>,
+    swap_chain_images: Option<Vec<Arc<SwapchainImage<winit::Window>>>>,
+    swap_chain_image_format: Option<Format>,
+    swap_chain_extent: Option<[u32; 2]>,
+
     events_loop: Option<winit::EventsLoop>,
 }

@@ -80,6 +102,7 @@ impl HelloTriangleApplication {
         self.create_surface();
         self.pick_physical_device();
         self.create_logical_device();
+        self.create_swap_chain();
     }

     fn create_instance(&mut self) {
@@ -153,7 +176,111 @@ impl HelloTriangleApplication {

     fn is_device_suitable(&self, device: &PhysicalDevice) -> bool {
         let indices = self.find_queue_families(device);
-        indices.is_complete()
+        let extensions_supported = Self::check_device_extension_support(device);
+
+        let swap_chain_adequate = if extensions_supported {
+                let capabilities = self.query_swap_chain_support(device);
+                !capabilities.supported_formats.is_empty() &&
+                    capabilities.present_modes.iter().next().is_some()
+            } else {
+                false
+            };
+
+        indices.is_complete() && extensions_supported && swap_chain_adequate
+    }
+
+    fn check_device_extension_support(device: &PhysicalDevice) -> bool {
+        let available_extensions = DeviceExtensions::supported_by_device(*device);
+        let device_extensions = device_extensions();
+        available_extensions.intersection(&device_extensions) == device_extensions
+    }
+
+    fn query_swap_chain_support(&self, device: &PhysicalDevice) -> Capabilities {
+        self.surface.as_ref().unwrap().capabilities(*device)
+            .expect("failed to get surface capabilities")
+    }
+
+    fn choose_swap_surface_format(available_formats: &[(Format, ColorSpace)]) -> (Format, ColorSpace) {
+        // NOTE: the 'preferred format' mentioned in the tutorial doesn't seem to be
+        // queryable in Vulkano (no VK_FORMAT_UNDEFINED enum)
+        *available_formats.iter()
+            .find(|(format, color_space)|
+                *format == Format::B8G8R8A8Unorm && *color_space == ColorSpace::SrgbNonLinear
+            )
+            .unwrap_or_else(|| &available_formats[0])
+    }
+
+    fn choose_swap_present_mode(available_present_modes: SupportedPresentModes) -> PresentMode {
+        if available_present_modes.mailbox {
+            PresentMode::Mailbox
+        } else if available_present_modes.immediate {
+            PresentMode::Immediate
+        } else {
+            PresentMode::Fifo
+        }
+    }
+
+    fn choose_swap_extent(&self, capabilities: &Capabilities) -> [u32; 2] {
+        if let Some(current_extent) = capabilities.current_extent {
+            return current_extent
+        } else {
+            let mut actual_extent = [WIDTH, HEIGHT];
+            actual_extent[0] = capabilities.min_image_extent[0]
+                .max(capabilities.max_image_extent[0].min(actual_extent[0]));
+            actual_extent[1] = capabilities.min_image_extent[1]
+                .max(capabilities.max_image_extent[1].min(actual_extent[1]));
+            actual_extent
+        }
+    }
+
+    fn create_swap_chain(&mut self) {
+        let instance = self.instance.as_ref().unwrap();
+        let physical_device = PhysicalDevice::from_index(instance, self.physical_device_index).unwrap();
+
+        let capabilities = self.query_swap_chain_support(&physical_device);
+
+        let surface_format = Self::choose_swap_surface_format(&capabilities.supported_formats);
+        let present_mode = Self::choose_swap_present_mode(capabilities.present_modes);
+        let extent = self.choose_swap_extent(&capabilities);
+
+        let mut image_count = capabilities.min_image_count + 1;
+        if capabilities.max_image_count.is_some() && image_count > capabilities.max_image_count.unwrap() {
+            image_count = capabilities.max_image_count.unwrap();
+        }
+
+        let image_usage = ImageUsage {
+            color_attachment: true,
+            .. ImageUsage::none()
+        };
+
+        let indices = self.find_queue_families(&physical_device);
+
+        let sharing: SharingMode = if indices.graphics_family != indices.present_family {
+            vec![self.graphics_queue.as_ref().unwrap(), self.present_queue.as_ref().unwrap()].as_slice().into()
+        } else {
+            self.graphics_queue.as_ref().unwrap().into()
+        };
+
+        let (swap_chain, images) = Swapchain::new(
+            self.device.as_ref().unwrap().clone(),
+            self.surface.as_ref().unwrap().clone(),
+            image_count,
+            surface_format.0, // TODO: color space?
+            extent,
+            1, // layers
+            image_usage,
+            sharing,
+            capabilities.current_transform,
+            CompositeAlpha::Opaque,
+            present_mode,
+            true, // clipped
+            None, // old_swapchain
+        ).expect("failed to create swap chain!");
+
+        self.swap_chain = Some(swap_chain);
+        self.swap_chain_images = Some(images);
+        self.swap_chain_image_format = Some(surface_format.0);
+        self.swap_chain_extent = Some(extent);
     }

     fn find_queue_families(&self, device: &PhysicalDevice) -> QueueFamilyIndices {
@@ -196,7 +323,7 @@ impl HelloTriangleApplication {
         // for us internally.

         let (device, mut queues) = Device::new(physical_device, &Features::none(),
-            &DeviceExtensions::none(), queue_families)
+            &device_extensions(), queue_families)
             .expect("failed to create logical device!");

         self.device = Some(device);
```
</details>

[Complete code](src/bin/06_swap_chain_creation.rs)

#### Image views
https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Image_views

We're skipping this section because image because image views are handled by Vulkano and can be accessed via the SwapchainImages created in the last section.

### Graphics pipeline basics
#### Introduction
https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics
<details>
<summary>Diff</summary>

```diff
--- a/06_swap_chain_creation.rs
+++ b/08_graphics_pipeline.rs
@@ -103,6 +103,7 @@ impl HelloTriangleApplication {
         self.pick_physical_device();
         self.create_logical_device();
         self.create_swap_chain();
+        self.create_graphics_pipeline();
     }

     fn create_instance(&mut self) {
@@ -283,6 +284,10 @@ impl HelloTriangleApplication {
         self.swap_chain_extent = Some(extent);
     }

+    fn create_graphics_pipeline(&mut self) {
+
+    }
```
</details>

[Complete code](src/bin/08_graphics_pipeline.rs)

#### Shader Modules
https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Shader_modules

Instead of compiling the shaders to SPIR-V manually and loading them at runtime, we'll use [vulkano-shader-derive](https://docs.rs/crate/vulkano-shader-derive/) to do the same at compile-time. Loading them at runtime is also possible, but a bit more invovled - see the [runtime shaders](https://github.com/vulkano-rs/vulkano/blob/master/examples/src/bin/runtime-shader.rs) example of Vulkano.
<details>
<summary>Diff</summary>

```diff
--- a/08_graphics_pipeline.rs
+++ b/09_shader_modules.rs
@@ -1,4 +1,6 @@
 extern crate vulkano;
+#[macro_use]
+extern crate vulkano_shader_derive;
 extern crate vulkano_win;
 extern crate winit;

@@ -285,7 +287,27 @@ impl HelloTriangleApplication {
     }

     fn create_graphics_pipeline(&mut self) {
+        #[allow(unused)]
+        mod vertex_shader {
+            #[derive(VulkanoShader)]
+            #[ty = "vertex"]
+            #[path = "src/bin/09_shader_base.vert"]
+            struct Dummy;
+        }
+
+        #[allow(unused)]
+        mod fragment_shader {
+            #[derive(VulkanoShader)]
+            #[ty = "fragment"]
+            #[path = "src/bin/09_shader_base.frag"]
+            struct Dummy;
+        }

+        let device = self.device.as_ref().unwrap();
+        let _vert_shader_module = vertex_shader::Shader::load(device.clone())
+            .expect("failed to create vertex shader module!");
+        let _frag_shader_module = fragment_shader::Shader::load(device.clone())
+            .expect("failed to create fragment shader module!");
     }
```
</details>

[Rust code](src/bin/09_shader_modules.rs) / [Vertex shader](src/bin/09_shader_base.vert) / [Fragment shader](src/bin/09_shader_base.frag)

#### Fixed functions
https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Fixed_functions

<details>
<summary>Diff</summary>

```diff
--- a/09_shader_modules.rs
+++ b/10_fixed_functions.rs
@@ -33,6 +33,11 @@ use vulkano::swapchain::{
 use vulkano::format::Format;
 use vulkano::image::{ImageUsage, swapchain::SwapchainImage};
 use vulkano::sync::SharingMode;
+use vulkano::pipeline::{
+    GraphicsPipeline,
+    vertex::BufferlessDefinition,
+    viewport::Viewport,
+};

 const WIDTH: u32 = 800;
 const HEIGHT: u32 = 600;
@@ -304,10 +309,35 @@ impl HelloTriangleApplication {
         }

         let device = self.device.as_ref().unwrap();
-        let _vert_shader_module = vertex_shader::Shader::load(device.clone())
+        let vert_shader_module = vertex_shader::Shader::load(device.clone())
             .expect("failed to create vertex shader module!");
-        let _frag_shader_module = fragment_shader::Shader::load(device.clone())
+        let frag_shader_module = fragment_shader::Shader::load(device.clone())
             .expect("failed to create fragment shader module!");
+
+        let swap_chain_extent = self.swap_chain_extent.unwrap();
+        let dimensions = [swap_chain_extent[0] as f32, swap_chain_extent[1] as f32];
+        let viewport = Viewport {
+            origin: [0.0, 0.0],
+            dimensions,
+            depth_range: 0.0 .. 1.0,
+        };
+
+        let _pipeline_builder = Arc::new(GraphicsPipeline::start()
+            .vertex_input(BufferlessDefinition {})
+            .vertex_shader(vert_shader_module.main_entry_point(), ())
+            .triangle_list()
+            .primitive_restart(false)
+            .viewports(vec![viewport]) // NOTE: also sets scissor to cover whole viewport
+            .depth_clamp(false)
+            // NOTE: there's an outcommented .rasterizer_discard() in Vulkano...
+            .polygon_mode_fill() // = default
+            .line_width(1.0) // = default
+            .cull_mode_back()
+            .front_face_clockwise()
+            // NOTE: no depth_bias here, but on pipeline::raster::Rasterization
+            .blend_pass_through() // = default
+            .fragment_shader(frag_shader_module.main_entry_point(), ())
+        );
     }
```
</details>

[Complete code](src/bin/10_fixed_functions.rs)

#### Render passes
https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Render_passes

<details>
<summary>Diff</summary>

```diff
--- a/10_fixed_functions.rs
+++ b/11_render_passes.rs
@@ -1,3 +1,4 @@
+#[macro_use]
 extern crate vulkano;
 #[macro_use]
 extern crate vulkano_shader_derive;
@@ -38,6 +39,9 @@ use vulkano::pipeline::{
     vertex::BufferlessDefinition,
     viewport::Viewport,
 };
+use vulkano::framebuffer::{
+    RenderPassAbstract,
+};

 const WIDTH: u32 = 800;
 const HEIGHT: u32 = 600;
@@ -90,6 +94,8 @@ struct HelloTriangleApplication {
     swap_chain_image_format: Option<Format>,
     swap_chain_extent: Option<[u32; 2]>,

+    render_pass: Option<Arc<RenderPassAbstract + Send + Sync>>,
+
     events_loop: Option<winit::EventsLoop>,
 }

@@ -110,6 +116,7 @@ impl HelloTriangleApplication {
         self.pick_physical_device();
         self.create_logical_device();
         self.create_swap_chain();
+        self.create_render_pass();
         self.create_graphics_pipeline();
     }

@@ -291,6 +298,23 @@ impl HelloTriangleApplication {
         self.swap_chain_extent = Some(extent);
     }

+    fn create_render_pass(&mut self) {
+        self.render_pass = Some(Arc::new(single_pass_renderpass!(self.device().clone(),
+            attachments: {
+                color: {
+                    load: Clear,
+                    store: Store,
+                    format: self.swap_chain.as_ref().unwrap().format(),
+                    samples: 1,
+                }
+            },
+            pass: {
+                color: [color],
+                depth_stencil: {}
+            }
+        ).unwrap()));
+    }
+
     fn create_graphics_pipeline(&mut self) {
         #[allow(unused)]
         mod vertex_shader {
@@ -421,6 +445,10 @@ impl HelloTriangleApplication {
     fn instance(&self) -> &Arc<Instance> {
         self.instance.as_ref().unwrap()
     }
+
+    fn device(&self) -> &Arc<Device> {
+        self.device.as_ref().unwrap()
+    }
 }
```
</details>

[Complete code](src/bin/11_render_passes.rs)
#### Conclusion
https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Conclusion

<details>
<summary>Diff</summary>

```diff
--- a/11_render_passes.rs
+++ b/12_graphics_pipeline_complete.rs
@@ -41,7 +41,9 @@ use vulkano::pipeline::{
 };
 use vulkano::framebuffer::{
     RenderPassAbstract,
+    Subpass,
 };
+use vulkano::descriptor::PipelineLayoutAbstract;

 const WIDTH: u32 = 800;
 const HEIGHT: u32 = 600;
@@ -77,6 +79,8 @@ impl QueueFamilyIndices {
     }
 }

+type ConcreteGraphicsPipeline = Arc<GraphicsPipeline<BufferlessDefinition, Box<PipelineLayoutAbstract + Send + Sync + 'static>, Arc<RenderPassAbstract + Send + Sync + 'static>>>;
+
 #[derive(Default)]
 struct HelloTriangleApplication {
     instance: Option<Arc<Instance>>,
@@ -95,6 +99,13 @@ struct HelloTriangleApplication {
     swap_chain_extent: Option<[u32; 2]>,

     render_pass: Option<Arc<RenderPassAbstract + Send + Sync>>,
+    // NOTE: We need to the full type of
+    // self.graphics_pipeline, because `BufferlessVertices` only
+    // works when the concrete type of the graphics pipeline is visible
+    // to the command buffer.
+    // TODO: check if can be simplified later in tutorial
+    // graphics_pipeline: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,
+    graphics_pipeline: Option<ConcreteGraphicsPipeline>,

     events_loop: Option<winit::EventsLoop>,
 }
@@ -346,12 +357,13 @@ impl HelloTriangleApplication {
             depth_range: 0.0 .. 1.0,
         };

-        let _pipeline_builder = Arc::new(GraphicsPipeline::start()
+        self.graphics_pipeline = Some(Arc::new(GraphicsPipeline::start()
             .vertex_input(BufferlessDefinition {})
             .vertex_shader(vert_shader_module.main_entry_point(), ())
             .triangle_list()
             .primitive_restart(false)
             .viewports(vec![viewport]) // NOTE: also sets scissor to cover whole viewport
+            .fragment_shader(frag_shader_module.main_entry_point(), ())
             .depth_clamp(false)
             // NOTE: there's an outcommented .rasterizer_discard() in Vulkano...
             .polygon_mode_fill() // = default
@@ -360,8 +372,10 @@ impl HelloTriangleApplication {
             .front_face_clockwise()
             // NOTE: no depth_bias here, but on pipeline::raster::Rasterization
             .blend_pass_through() // = default
-            .fragment_shader(frag_shader_module.main_entry_point(), ())
-        );
+            .render_pass(Subpass::from(self.render_pass.as_ref().unwrap().clone(), 0).unwrap())
+            .build(device.clone())
+            .unwrap()
+        ));
     }
```
</details>

[Complete code](src/bin/12_graphics_pipeline_complete.rs)


### Drawing (*TODO*)
#### Framebuffers
https://vulkan-tutorial.com/Drawing_a_triangle/Drawing/Framebuffers

<details>
<summary>Diff</summary>

```diff
--- a/12_graphics_pipeline_complete.rs
+++ b/13_framebuffers.rs
@@ -42,6 +42,8 @@ use vulkano::pipeline::{
 use vulkano::framebuffer::{
     RenderPassAbstract,
     Subpass,
+    FramebufferAbstract,
+    Framebuffer,
 };
 use vulkano::descriptor::PipelineLayoutAbstract;

@@ -107,6 +109,8 @@ struct HelloTriangleApplication {
     // graphics_pipeline: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,
     graphics_pipeline: Option<ConcreteGraphicsPipeline>,

+    swap_chain_framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
+
     events_loop: Option<winit::EventsLoop>,
 }

@@ -129,6 +133,7 @@ impl HelloTriangleApplication {
         self.create_swap_chain();
         self.create_render_pass();
         self.create_graphics_pipeline();
+        self.create_framebuffers();
     }

     fn create_instance(&mut self) {
@@ -378,6 +383,17 @@ impl HelloTriangleApplication {
         ));
     }

+    fn create_framebuffers(&mut self) {
+        self.swap_chain_framebuffers = self.swap_chain_images.as_ref().unwrap().iter()
+            .map(|image| {
+                let fba: Arc<FramebufferAbstract + Send + Sync> = Arc::new(Framebuffer::start(self.render_pass.as_ref().unwrap().clone())
+                    .add(image.clone()).unwrap()
+                    .build().unwrap());
+                fba
+            }
+        ).collect::<Vec<_>>();
+    }
+
```
</details>

[Complete code](src/bin/13_framebuffers.rs)

#### Command buffers
https://vulkan-tutorial.com/Drawing_a_triangle/Drawing/Command_buffers

We're skipping the first part because Vulkano maintains a [`StandardCommandPool`].(https://docs.rs/vulkano/0.10.0/vulkano/command_buffer/pool/standard/struct.StandardCommandPool.html)

<details>
<summary>Diff</summary>

```diff
--- a/13_framebuffers.rs
+++ b/14_command_buffers.rs
@@ -37,6 +37,7 @@ use vulkano::sync::SharingMode;
 use vulkano::pipeline::{
     GraphicsPipeline,
     vertex::BufferlessDefinition,
+    vertex::BufferlessVertices,
     viewport::Viewport,
 };
 use vulkano::framebuffer::{
@@ -46,6 +47,11 @@ use vulkano::framebuffer::{
     Framebuffer,
 };
 use vulkano::descriptor::PipelineLayoutAbstract;
+use vulkano::command_buffer::{
+    AutoCommandBuffer,
+    AutoCommandBufferBuilder,
+    DynamicState,
+};

 const WIDTH: u32 = 800;
 const HEIGHT: u32 = 600;
@@ -111,6 +117,8 @@ struct HelloTriangleApplication {

     swap_chain_framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,

+    command_buffers: Vec<Arc<AutoCommandBuffer>>,
+
     events_loop: Option<winit::EventsLoop>,
 }

@@ -134,6 +142,7 @@ impl HelloTriangleApplication {
         self.create_render_pass();
         self.create_graphics_pipeline();
         self.create_framebuffers();
+        self.create_command_buffers();
     }

     fn create_instance(&mut self) {
@@ -394,6 +403,27 @@ impl HelloTriangleApplication {
         ).collect::<Vec<_>>();
     }

+    fn create_command_buffers(&mut self) {
+        let queue_family = self.graphics_queue.as_ref().unwrap().family();
+        let graphics_pipeline = self.graphics_pipeline.as_ref().unwrap();
+        self.command_buffers = self.swap_chain_framebuffers.iter()
+            .map(|framebuffer| {
+                let vertices = BufferlessVertices { vertices: 3, instances: 1 };
+                Arc::new(AutoCommandBufferBuilder::primary_simultaneous_use(self.device().clone(), queue_family)
+                    .unwrap()
+                    .begin_render_pass(framebuffer.clone(), false, vec![[0.0, 0.0, 0.0, 1.0].into()])
+                    .unwrap()
+                    .draw(graphics_pipeline.clone(), &DynamicState::none(),
+                        vertices, (), ())
+                    .unwrap()
+                    .end_render_pass()
+                    .unwrap()
+                    .build()
+                    .unwrap())
+            })
+            .collect();
+    }
+
```
</details>

[Complete code](src/bin/14_command_buffers.rs)

#### Rendering and presentation

### Swapchain recreation (*TODO*)
[Complete code](src/main.rs)

## Vertex buffers (*TODO*)
## Uniform buffers (*TODO*)
## Texture mapping (*TODO*)
## Depth buffering (*TODO*)
## Loading models (*TODO*)
## Generating Mipmaps (*TODO*)
## Multisampling (*TODO*)

---
<details>
<summary>Diff</summary>

```diff
```
</details>

[Complete code](src/bin/XXX.rs)
