
use anyhow::Result;
use wgpu::util::DeviceExt;

use crate::renderer::{Vertex, Renderer};

// Just define a basic quad for tile images.
pub const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 0.0], },
    Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0], },
    Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 1.0], },
    Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 0.0], },
];

pub const NUM_VERTICES: u32 = VERTICES.len() as u32;

// Having an index buffer saves a little bit on memory bandwidth between cpu and gpu.
// In this case specifically I don't think it really saves that much, it is saving something
// like ~8 bytes per image so not super worthwhile, but index buffers have other benefits like 
// cache locality and seem like better practice IMO.
pub const INDICES: &[u16] = &[
    0, 1, 2,
    2, 3, 0,
];

pub const NUM_INDICES: u32 = INDICES.len() as u32;

// Just separating this out so we don't have to keep sending vertices/indices every draw call.
pub struct ImageMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl ImageMesh {
    pub fn new(device: &wgpu::Device) -> Result<ImageMesh> {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        
        Ok(ImageMesh { vertex_buffer, index_buffer })
    }
}

pub struct Image {
    pub texture: crate::renderer::Texture,
    pub bind_group: wgpu::BindGroup,
}

pub struct ImageHandle(pub usize);

impl Renderer {
    pub fn create_image(&mut self, texture: crate::renderer::Texture) -> Result<ImageHandle> {
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let image = Image {
            bind_group,
            texture,
        };

        let index = self.images.push(image);
        Ok(ImageHandle(index))
    }
}