

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
}

/*
impl Renderer {
    async fn new(window: &Window) -> Self {

    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {

    }

    fn input(&mut self, event: &WindowEvent) {

    }

    fn update(&mut self) {

    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        
    }
}
*/