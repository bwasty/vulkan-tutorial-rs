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
cargo new vulkan-tutorial-rs
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

[Rust code](src/bin/00_base_code.rs)


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

*TODO*

#### Physical devices and queue families
https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Physical_devices_and_queue_families

*TODO*

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
