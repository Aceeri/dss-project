
use wgpu::util::DeviceExt;

use anyhow::Result;

use crate::{
    renderer::{Vertex, Texture, RenderContext},
    util::{ReuseVec, ManagedBuffer},
};
use super::{sprite, SpriteTexture, SpriteInstance, SpriteMesh};

#[derive(Debug, Clone)]
pub struct SpriteTextureId(usize);

impl SpriteTextureId {
    fn id(&self) -> usize {
        self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SpriteInstanceId(usize);

impl SpriteInstanceId {
    fn id(&self) -> usize {
        self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SpriteId(usize);

impl SpriteId {
    fn id(&self) -> usize {
        self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Sprite {
    pub instance: SpriteInstanceId,
    pub texture: SpriteTextureId,
}

pub struct SpritePass {
    shader: wgpu::ShaderModule,

    pipeline: wgpu::RenderPipeline,
    pipeline_layout: wgpu::PipelineLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,

    textures: ReuseVec<SpriteTexture>,
    instances: ManagedBuffer<SpriteInstance>,
    mesh: SpriteMesh,

    // Pair of instance and sprite.
    //
    // Separated out so instancing could be done easier if wanted.
    sprites: ReuseVec<Sprite>,
}

impl SpritePass {
    pub fn new(context: &RenderContext) -> Result<Self> {
        let shader = context.device().create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let sprite_instance_buffer = context.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Instance Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let sprite_mesh = SpriteMesh::new(context.device())?;

        let texture_bind_group_layout =
            context.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: Some("sprite_texture_bind_group_layout"),
            });

        let pipeline_layout =
            context.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Sprite Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &context.camera_bind_group_layout()],
                push_constant_ranges: &[],
            });

        let vertex_state = wgpu::VertexState {
            module: &shader,
            entry_point: "main",
            buffers: &[Vertex::desc(), SpriteInstance::desc()],
        };

        let fragment_state = wgpu::FragmentState {
            module: &shader,
            entry_point: "main",
            targets: &[wgpu::ColorTargetState {
                format: context.config().format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        let pipeline = context.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
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

        let sprite_instances = ManagedBuffer::new(
            context.device(),
            Some("sprite instance buffer".to_owned()),
            wgpu::BufferUsages::VERTEX,
        );

        Ok(Self {
            shader,


            pipeline,
            pipeline_layout,

            texture_bind_group_layout,

            textures: ReuseVec::new(),
            mesh: sprite_mesh,
            instances: sprite_instances,

            sprites: ReuseVec::new(),
        })
    }

    pub fn render(&mut self, context: &RenderContext) {

        context.encoder()
        encoder.set_pipeline(&self.pipeline);
        encoder.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_bind_group(1, &context.camera_bind_group(), &[]);

        // Ideally we would batch this into a single draw call by using texture atlases/arrays but
        // for the sake of simplicity going to just do one draw call per tile image.
        //
        // Texture atlases have their own problems, and texture arrays aren't supported in older GPUs so probably
        // can't be used in this case.
        //
        // Could also have some fun with multithreading these draw calls although I don't know how much performance
        // that would really save in this case.

        // Render Images
        render_pass.set_vertex_buffer(0, sprite_mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(
            self.image_mesh.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );

        for sprite in self.sprites.iter() {
            let texture = self.textures.get(sprite.texture.id());
            if let Some(texture) = texture {
                render_pass.set_bind_group(0, &texture.bind_group, &[]);
                render_pass.draw_indexed(
                    0..sprite::NUM_INDICES,
                    0,
                    sprite.instance.id() as u32..sprite.instance.id() as u32 + 1,
                );
            }
        }

        encoder.finish(&wgpu::RenderBundleDescriptor {
            label: Some("sprite"),
        })
    }

    pub fn create_image(&mut self, device: &wgpu::Device, texture: crate::renderer::Texture) -> SpriteHandle {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        let sprite = Sprite {
            bind_group,
            texture,
        };

        let index = self.sprites.push(image);
        SpriteHandle(index)
    }
}