
#[cfg(feature = "dx11")]
extern crate gfx_backend_dx11 as back;
#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(feature = "gl")]
extern crate gfx_backend_gl as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;
#[cfg(not(any(
    feature = "vulkan",
    feature = "dx11",
    feature = "dx12",
    feature = "metal",
    feature = "gl",
)))]
extern crate gfx_backend_empty as back;

use gfx_hal::{
    format::{AsFormat, ChannelType, Rgba8Srgb as ColorFormat, Swizzle},
    image as i, memory as m, pass,
    pass::Subpass,
    pool,
    prelude::*,
    pso,
    pso::{PipelineStage, ShaderStageFlags, VertexInputRate},
    queue::QueueGroup,
    window,
};

use gfx::InstanceBuffer;
use gfx_hal::Instance;
use winit::{
    window::{ Window, WindowBuilder },
    event_loop::EventLoop,
    dpi::{LogicalSize, PhysicalSize},
};


fn main() {
    let instance = back::Instance::create("dss project", 1).expect("Failed to create an instance");

    let adapter = {
        let mut adapters = instance.enumerate_adapters();
        for adapter in &adapters {
            println!("{:?}", adapter.info);
        }
        adapters.remove(0)
    };

    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_min_inner_size(LogicalSize::new(64.0, 64.0))
        .with_inner_size(PhysicalSize::new(64.0, 64.0))
        .with_title("DSS Project".to_string());
    
    let window = window_builder.build(&event_loop).unwrap();


    let surface = unsafe {
        instance
            .create_surface(&window)
            .expect("Failed to create surface")
    };

    let mut renderer = streaming_home::renderer::Renderer::new(instance, surface, adapter);
    renderer.render();

    event_loop.run(move |event, _, control_flow| {
            *control_flow = winit::event_loop::ControlFlow::Wait;

            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit
                    }
                    winit::event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = winit::event_loop::ControlFlow::Exit,
                    winit::event::WindowEvent::Resized(dims) => {
                        println!("resized to {:?}", dims);
                        renderer.set_dimensions(window::Extent2D {
                            width: dims.width,
                            height: dims.height,
                        });
                        renderer.recreate_swapchain();
                    }
                    _ => {}
                },
                winit::event::Event::RedrawEventsCleared => {
                    renderer.render();
                }
                _ => {}
            }
        });
}
