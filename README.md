# vulkan-tutorial-rs
Rust version of https://github.com/Overv/VulkanTutorial using [Vulkano](http://vulkano.rs/).

**Goal**: Rust port with code structure as similar as possible to the original C++, so the original tutorial can easily be followed (similar to [learn-opengl-rs](https://github.com/bwasty/learn-opengl-rs)).

**Current State**: The first major part, `Drawing a triangle`, is complete.

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
   * [Drawing](#drawing)
      * [Framebuffers](#framebuffers)
      * [Command buffers](#command-buffers)
      * [Rendering and presentation](#rendering-and-presentation)
   * [Swapchain recreation](#swapchain-recreation)
* [Vertex buffers (<em>TODO</em>)](#vertex-buffers-todo)
* [Uniform buffers (<em>TODO</em>)](#uniform-buffers-todo)
* [Texture mapping (<em>TODO</em>)](#texture-mapping-todo)
* [Depth buffering (<em>TODO</em>)](#depth-buffering-todo)
* [Loading models (<em>TODO</em>)](#loading-models-todo)
* [Generating Mipmaps (<em>TODO</em>)](#generating-mipmaps-todo)
* [Multisampling (<em>TODO</em>)](#multisampling-todo)

## Introduction
This tutorial consists of the the ported code and notes about the differences between the original C++ and the Rust code.
The [explanatory texts](https://vulkan-tutorial.com/Introduction) generally apply equally, although the Rust version is often shorter due to the use of [Vulkano](http://vulkano.rs/), a safe wrapper around the Vulkan API with some convencience functionality (the final triangle example is about 600 lines, compared to 950 lines in C++).

If you prefer a lower-level API closer to the Vulkan C API, have a look at [Ash](https://github.com/MaikKlein/ash) and [vulkan-tutorial-rust](https://github.com/Usami-Renko/vulkan-tutorial-rust).

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

On macOS, copy [mac-env.sh](mac-env.sh), adapt the `VULKAN_SDK` path if necessary and `source` the file in your terminal. See also [vulkano-rs/vulkano#macos-and-ios-setup](https://github.com/vulkano-rs/vulkano#macos-and-ios-setup).

## Drawing a triangle
### Setup
#### Base code
https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Base_code
##### General structure
```rust
extern crate vulkano;

struct HelloTriangleApplication {

}

impl HelloTriangleApplication {
    pub fn initialize() -> Self {
        Self {

        }
    }

    fn main_loop(&mut self) {

    }
}

fn main() {
    let mut app = HelloTriangleApplication::new();
    app.main_loop();
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

use winit::{WindowBuilder, dpi::LogicalSize};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

struct HelloTriangleApplication {
    events_loop: EventsLoop,
}
```
```rust
    pub fn initialize() -> Self {
        let events_loop = Self::init_window();

        Self {
            events_loop,
        }
    }

    fn init_window() -> EventsLoop {
        let events_loop = EventsLoop::new();
        let _window = WindowBuilder::new()
            .with_title("Vulkan")
            .with_dimensions(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
            .build(&events_loop);
        events_loop
    }
```
```rust
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
```

[Complete code](src/bin/00_base_code.rs)


#### Instance
https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Instance

Cargo.toml:
```
vulkano-win = "0.10.0"
```
main.rs:
```rust
extern crate vulkano_win;
```
```rust
use std::sync::Arc;
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
    events_loop: EventsLoop,
}
```
```rust
     pub fn initialize() -> Self {
        let instance = Self::create_instance();
         let events_loop = Self::init_window();

         Self {
            instance,
             events_loop,
         }
     }
```
```rust
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
```

[Complete code](src/bin/01_instance_creation.rs)

#### Validation layers
https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Validation_layers

[Diff](src/bin/02_validation_layers.rs.diff)

[Complete code](src/bin/02_validation_layers.rs)


#### Physical devices and queue families
https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Physical_devices_and_queue_families

[Diff](src/bin/03_physical_device_selection.rs.diff)

[Complete code](src/bin/03_physical_device_selection.rs)


#### Logical device and queues
https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Logical_device_and_queues

[Diff](src/bin/04_logical_device.rs.diff)

[Complete code](src/bin/04_logical_device.rs)

### Presentation

#### Window surface
https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Window_surface

[Diff](src/bin/05_window_surface.rs.diff)

[Complete code](src/bin/05_window_surface.rs)

#### Swap chain
https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Swap_chain

[Diff](src/bin/06_swap_chain_creation.rs.diff)

[Complete code](src/bin/06_swap_chain_creation.rs)

#### Image views
https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Image_views

We're skipping this section because image views are handled by Vulkano and can be accessed via the `SwapchainImage`s created in the last section.

### Graphics pipeline basics
#### Introduction
https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics

[Diff](src/bin/08_graphics_pipeline.rs.diff)

[Complete code](src/bin/08_graphics_pipeline.rs)

#### Shader Modules
https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Shader_modules

Instead of compiling the shaders to SPIR-V manually and loading them at runtime, we'll use [vulkano-shader-derive](https://docs.rs/crate/vulkano-shader-derive/) to do the same at compile-time. Loading them at runtime is also possible, but a bit more invovled - see the [runtime shaders](https://github.com/vulkano-rs/vulkano/blob/master/examples/src/bin/runtime-shader.rs) example of Vulkano.

[Diff](src/bin/09_shader_modules.rs.diff)

[Rust code](src/bin/09_shader_modules.rs) / [Vertex shader](src/bin/09_shader_base.vert) / [Fragment shader](src/bin/09_shader_base.frag)

#### Fixed functions
https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Fixed_functions

[Diff](src/bin/10_fixed_functions.rs.diff)

[Complete code](src/bin/10_fixed_functions.rs)

#### Render passes
https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Render_passes

[Diff](src/bin/11_render_passes.rs.diff)

[Complete code](src/bin/11_render_passes.rs)

#### Conclusion
https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Conclusion

[Diff](src/bin/12_graphics_pipeline_complete.rs.diff)

[Complete code](src/bin/12_graphics_pipeline_complete.rs)


### Drawing
#### Framebuffers
https://vulkan-tutorial.com/Drawing_a_triangle/Drawing/Framebuffers

[Diff](src/bin/13_framebuffers.rs.diff)

[Complete code](src/bin/13_framebuffers.rs)

#### Command buffers
https://vulkan-tutorial.com/Drawing_a_triangle/Drawing/Command_buffers

We're skipping the first part because Vulkano maintains a [`StandardCommandPool`](https://docs.rs/vulkano/0.10.0/vulkano/command_buffer/pool/standard/struct.StandardCommandPool.html).

[Diff](src/bin/14_command_buffers.rs.diff)

[Complete code](src/bin/14_command_buffers.rs)

#### Rendering and presentation
https://vulkan-tutorial.com/Drawing_a_triangle/Drawing/Rendering_and_presentation

[Diff](src/bin/15_hello_triangle.rs.diff)

[Complete code](src/bin/15_hello_triangle.rs)

### Swapchain recreation
https://vulkan-tutorial.com/Drawing_a_triangle/Swap_chain_recreation

[Diff](src/bin/16_swap_chain_recreation.rs.diff)

[Complete code](src/bin/16_swap_chain_recreation.rs)

## Vertex buffers (*TODO*)
## Uniform buffers (*TODO*)
## Texture mapping (*TODO*)
## Depth buffering (*TODO*)
## Loading models (*TODO*)
## Generating Mipmaps (*TODO*)
## Multisampling (*TODO*)
