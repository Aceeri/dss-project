
use anyhow::Result;
use wgpu::util::DeviceExt;
use winit::{
    event::WindowEvent,
    window::Window,
};

use glam::{Mat4, Vec2, Vec3};

use std::mem;

use super::{Camera, CameraUniform, SpritePass, Sprite, SpriteInstance, SpriteId, SpriteMesh, Texture};
use crate::util::ReuseVec;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct Renderer {
    pub context: RenderContext, 
    pub sprite_pass: SpritePass,
    pub test_pass: SpritePass,
    //text_pass: TextPass,
}

impl Renderer {
    pub async fn new(window: &Window) -> Result<Self> {
        let context = RenderContext::new(window).await?;
        let sprite_pass = SpritePass::new(&context)?;
        let mut test_pass = SpritePass::new(&context)?;

        let fallback_bytes = include_bytes!("test.png");
        let texture = Texture::from_bytes(&context.device(), &context.queue(), fallback_bytes, "fallback.png")?;
        let texture_id = test_pass.add_texture(context.device(), texture);
        let instance_id = test_pass.add_instance(SpriteInstance { position: [0.5, 0.5, 1.0], size: [5.0, 5.0] });
        let sprite_id = test_pass.add_sprite(texture_id, instance_id);
        Ok(Self {
            context,
            sprite_pass,
            test_pass,
        })
    }

    pub fn context(&self) -> &RenderContext {
        &self.context
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.context().size()
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.context.resize(new_size);
    }

    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        /*
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color = wgpu::Color {
                    r: position.x / self.size.width as f64,
                    g: position.y / self.size.height as f64,
                    b: 0.5,
                    a: 1.0,
                };

                true
            }
            _ => false,
        }
        */
        false
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.context.surface().get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .context
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            // Clear color render pass.
            let mut clear_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(*self.context.clear_color()),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.context.depth_texture().view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
        }

        self.sprite_pass.render(&self.context, &mut encoder, &view);
        self.test_pass.render(&self.context, &mut encoder, &view);

        self.context.queue().submit(std::iter::once(encoder.finish()));

        // Need to present the wgpu frame now instead of just dropping.
        frame.present();

        Ok(())
    }

}

// Commonly shared state between render passes.
pub struct RenderContext {
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    sample_count: u32,

    depth_texture: Texture,

    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_bind_group_layout: wgpu::BindGroupLayout,

    clear_color: wgpu::Color,
}

impl RenderContext {
    pub async fn new(window: &Window) -> Result<Self> {
        let size = window.inner_size();
        let sample_count = 1;

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(anyhow!("could not find adapter that meet requirements"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await?;

        let format = surface.get_preferred_format(&adapter)
            .ok_or(anyhow!("surface is incompatible with adapter"))?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera = super::Camera::new(config.width as f32, config.height as f32);
        let mut camera_uniform = super::CameraUniform::new();
        camera_uniform.set_view_matrix(&camera);

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        /*
        let fallback_bytes = include_bytes!("test.png");
        let fallback_texture = crate::renderer::Texture::from_bytes(&device, &queue, fallback_bytes, "fallback.png").expect("created texture");
        let fallback_image_handle = self.create_image(fallback_texture);
        */

        let clear_color = wgpu::Color {
            r: 0.0,
            g: 0.005,
            b: 0.0,
            a: 1.0,
        };

        Ok(Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            config,
            size,
            sample_count,

            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,

            depth_texture,

            clear_color,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;

            // Fix aspect ratio for our camera.
            self.camera = Camera::new(new_size.width as f32, new_size.height as f32);
            self.camera_uniform.set_view_matrix(&self.camera);

            // Depth texture is the same size as our screen so we need to resize it.
            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.config, "depth_texture");

            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );
            self.surface.configure(&self.device, &self.config);
        }
    }

    // Getters for encapsulation purposes.
    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    pub fn depth_texture(&self) -> &Texture {
        &self.depth_texture
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn camera_uniform(&self) -> &CameraUniform {
        &self.camera_uniform
    }

    pub fn camera_bind_group(&self) -> &wgpu::BindGroup {
        &self.camera_bind_group
    }

    pub fn camera_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bind_group_layout
    }

    pub fn clear_color(&self) -> &wgpu::Color {
        &self.clear_color
    }
}
