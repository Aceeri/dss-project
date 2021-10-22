use anyhow::Result;
use wgpu::util::DeviceExt;
use winit::{
    event::{KeyboardInput, VirtualKeyCode, WindowEvent},
    window::Window,
};

use glam::{Mat4, Vec3};

use std::{collections::HashMap, mem};

use crate::renderer::{Image, ImageHandle, ImageMesh, Texture};
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

#[derive(Copy, Clone, Debug)]
pub struct InstanceHandle(usize);

#[derive(Copy, Clone, Debug)]
pub struct ImageInstanceHandle(usize);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub position: [f32; 3],
    pub size: [f32; 2],
}

impl Instance {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

// An orthographic camera, mostly just used here for keeping scaling of objects tidy, but could be useful to swap out later
// for a perspective camera to make things look more fancy.
#[derive(Debug, Clone)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        let aspect_ratio = width / height;
        let scaling = 5.0; // Just make things a bit easier to work with.
        Self {
            // Back up 1 so we can actually see the images.
            eye: (0.0, 0.0, 1.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vec3::Y,

            // Scale vertically and have it "anchor" at the top left.
            left: 0.0,
            right: aspect_ratio * scaling,
            top: 0.0,
            bottom: -1.0 * scaling,

            near: 0.0,
            far: 100.0,
        }
    }

    pub fn build_view_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let ortho = Mat4::orthographic_rh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        );
        ortho * view
    }
}

// Something that we will actually send to the GPU for the shaders to use.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_matrix: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_matrix: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn set_view_matrix(&mut self, camera: &Camera) {
        self.view_matrix = camera.build_view_matrix().to_cols_array_2d();
    }
}

#[derive(Debug, Clone)]
pub struct ImageInstance {
    image: ImageHandle,
    instance: InstanceHandle,
}

pub struct Renderer {
    pub(crate) surface: wgpu::Surface,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,

    pub(crate) images: ReuseVec<Image>,
    pub(crate) image_mesh: ImageMesh,

    pub(crate) instances: ReuseVec<Instance>,
    pub(crate) instance_buffer: wgpu::Buffer,
    update_instance_buffer: bool,
    expand_instance_buffer: bool,

    image_instances: ReuseVec<ImageInstance>,

    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,

    depth_texture: Texture,

    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    clear_color: wgpu::Color,
}

impl Renderer {
    pub async fn new(window: &Window) -> Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

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

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let vertex_state = wgpu::VertexState {
            module: &shader,
            entry_point: "main",
            buffers: &[Vertex::desc(), Instance::desc()],
        };

        let fragment_state = wgpu::FragmentState {
            module: &shader,
            entry_point: "main",
            targets: &[wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                //blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        };

        let primitive_state = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            clamp_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        };

        let depth_stencil_state = wgpu::DepthStencilState {
            format: crate::renderer::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: vertex_state,
            fragment: Some(fragment_state),
            primitive: primitive_state,
            depth_stencil: Some(depth_stencil_state),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        let camera = Camera::new(config.width as f32, config.height as f32);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.set_view_matrix(&camera);

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

        let image_mesh = ImageMesh::new(&device)?;

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,

            images: ReuseVec::new(),
            image_mesh,
            texture_bind_group_layout,

            image_instances: ReuseVec::new(),

            instances: ReuseVec::new(),
            instance_buffer,
            update_instance_buffer: false,
            expand_instance_buffer: false,

            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,

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
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let ops = wgpu::Operations {
                load: wgpu::LoadOp::Clear(self.clear_color),
                store: true,
            };

            let color_attachment = wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: ops,
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[color_attachment],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            if self.expand_instance_buffer {
                // Probably should do some sort of amortized doubling of this buffer here similar to Vecs.
                self.instance_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Instance Buffer"),
                            contents: bytemuck::cast_slice(self.instances.current().as_slice()),
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        });
            } else if self.update_instance_buffer {
                self.queue.write_buffer(
                    &self.instance_buffer,
                    0,
                    bytemuck::cast_slice(self.instances.current().as_slice()),
                );
            }

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            // Ideally we would batch this into a single draw call by using texture atlases/arrays but
            // for the sake of simplicity going to just do one draw call per tile image.
            //
            // Texture atlases have their own problems, and texture arrays aren't supported in older GPUs so probably
            // can't be used in this case.
            //
            // Could also have some fun with multithreading these draw calls although I don't know how much performance
            // that would really save in this case.

            // Render Images
            render_pass.set_vertex_buffer(0, self.image_mesh.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(
                self.image_mesh.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            for image_instance in self.image_instances.iter() {
                let image = self.images.get(image_instance.image.0);
                if let Some(image) = image {
                    render_pass.set_bind_group(0, &image.bind_group, &[]);
                    render_pass.draw_indexed(
                        0..crate::renderer::sprite::NUM_INDICES,
                        0,
                        image_instance.instance.0 as u32..image_instance.instance.0 as u32 + 1,
                    );
                }
            }

            // TODO: Render text
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        // Need to present the wgpu frame now instead of just dropping.
        frame.present();

        Ok(())
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    // TODO: removing instances.

    pub fn create_instance(&mut self, instance: Instance) -> InstanceHandle {
        let index = self.instances.push(instance);

        self.expand_instance_buffer = true;

        InstanceHandle(index)
    }

    pub fn create_image_instance(
        &mut self,
        image_handle: ImageHandle,
        instance_handle: InstanceHandle,
    ) -> ImageInstanceHandle {
        let index = self.image_instances.push(ImageInstance {
            image: image_handle,
            instance: instance_handle,
        });

        ImageInstanceHandle(index)
    }

    pub fn set_instance(&mut self, handle: InstanceHandle, new_instance: Instance) {
        match self.instances.get_mut(handle.0) {
            Some(instance) => {
                *instance = new_instance;
                self.update_instance_buffer = true;
            }
            None => {}
        }
    }

    pub fn set_image_instance_position(
        &mut self,
        handle: ImageInstanceHandle,
        new_instance: Instance,
    ) {
        let image_instance = self.image_instances.get(handle.0).cloned();
        if let Some(image_instance) = image_instance {
            self.set_instance(image_instance.instance, new_instance);
        }
    }
}
