use anyhow::Result;
use wgpu::util::DeviceExt;

use crate::renderer::{Vertex};

use std::mem;

// Just define a basic quad for tile images.
pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, 0.5, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.0],
        tex_coords: [1.0, 0.0],
    },
];

pub const NUM_VERTICES: u32 = VERTICES.len() as u32;

// Having an index buffer saves a little bit on memory bandwidth between cpu and gpu.
// In this case specifically I don't think it really saves that much, it is saving something
// like ~8 bytes per image so not super worthwhile, but index buffers have other benefits like
// cache locality and seem like better practice IMO.
pub const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

pub const NUM_INDICES: u32 = INDICES.len() as u32;

// Just separating this out so we don't have to keep sending vertices/indices every draw call.
pub struct SpriteMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl SpriteMesh {
    pub fn new(device: &wgpu::Device) -> Result<Self> {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Ok(Self {
            vertex_buffer,
            index_buffer,
        })
    }
}

pub struct SpriteTexture {
    pub texture: crate::renderer::Texture,
    pub bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteInstance {
    pub position: [f32; 3],
    pub size: [f32; 2],
}

impl SpriteInstance {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
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
