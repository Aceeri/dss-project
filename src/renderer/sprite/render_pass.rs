use anyhow::Result;

use super::{sprite, SpriteInstance, SpriteMesh, SpriteTexture};
use crate::{
    renderer::{RenderContext, Texture, Vertex},
    util::{IdIndex, ManagedBuffer, ReuseVec},
};

#[derive(Copy, Clone, Debug)]
pub struct SpriteTextureId(usize);

impl SpriteTextureId {
    fn id(&self) -> usize {
        self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SpriteInstanceId(usize);

impl IdIndex for SpriteInstanceId {
    fn id(&self) -> usize {
        self.0
    }
    fn from_index(index: usize) -> Self {
        Self(index)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SpriteId(usize);

impl IdIndex for SpriteId {
    fn id(&self) -> usize {
        self.0
    }
    fn from_index(index: usize) -> Self {
        Self(index)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Sprite {
    pub instance: SpriteInstanceId,
    pub texture: SpriteTextureId,
}

pub struct SpritePass {
    pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,

    textures: ReuseVec<SpriteTexture>,
    instances: ManagedBuffer<SpriteInstance, SpriteInstanceId>,
    mesh: SpriteMesh,

    // Pair of instance and sprite.
    //
    // Separated out so instancing could be done easier if wanted.
    sprites: ReuseVec<Sprite>,
}

impl SpritePass {
    pub fn new(context: &RenderContext) -> Result<Self> {
        let shader = context
            .device()
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some("SpritePass::shader.wgsl"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        let sprite_mesh = SpriteMesh::new(context.device())?;

        let texture_bind_group_layout =
            context
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    label: Some("SpritePass::texture_bind_group_layout"),
                });

        let pipeline_layout =
            context
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("SpritePass:pipeline_layout"),
                    bind_group_layouts: &[
                        &texture_bind_group_layout,
                        &context.camera_bind_group_layout(),
                    ],
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

        let pipeline = context
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("SpritePass::pipeline"),
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
            Some("SpritePass::sprite_instances"),
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        Ok(Self {
            pipeline,

            texture_bind_group_layout,

            textures: ReuseVec::new(),
            mesh: sprite_mesh,
            instances: sprite_instances,

            sprites: ReuseVec::new(),
        })
    }

    pub fn render(
        &mut self,
        context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        self.instances
            .update_buffer(context.device(), context.queue());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("SpritePass::render_pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &context.depth_texture().view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(1, &context.camera_bind_group(), &[]);

        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instances.buffer_slice(..));
        render_pass.set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        // Ideally we would batch this into a single draw call by using texture atlases/arrays but
        // for the sake of simplicity going to just do one draw call per tile image.
        //
        // Texture atlases have their own problems, and texture arrays aren't supported in older GPUs so probably
        // can't be used in this case.

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
    }

    // Add a new texture to the sprite pass.
    pub fn add_texture(&mut self, device: &wgpu::Device, texture: Texture) -> SpriteTextureId {
        let sprite_texture =
            SpritePass::bind_sprite_texture(device, &self.texture_bind_group_layout, texture);
        let index = self.textures.push(sprite_texture);
        SpriteTextureId(index)
    }

    pub fn bind_sprite_texture(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        texture: Texture,
    ) -> SpriteTexture {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: layout,
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
            label: Some("SpritePass::sprite_texture_bind_group"),
        });

        SpriteTexture {
            bind_group,
            texture,
        }
    }

    pub fn add_instance(&mut self, instance: SpriteInstance) -> SpriteInstanceId {
        self.instances.push(instance)
    }

    pub fn add_sprite(&mut self, texture: SpriteTextureId, instance: SpriteInstanceId) -> SpriteId {
        let index = self.sprites.push(Sprite { texture, instance });
        SpriteId(index)
    }

    pub fn set_instance(&mut self, id: SpriteInstanceId, new_instance: SpriteInstance) {
        self.instances.set(id, new_instance);
    }

    pub fn set_sprite_instance(&mut self, handle: SpriteId, new_instance: SpriteInstance) {
        let sprite = self.sprites.get(handle.0).cloned();
        if let Some(sprite) = sprite {
            self.set_instance(sprite.instance, new_instance);
        }
    }

    pub fn fallback_texture(&self, context: &RenderContext) -> Texture {
        let fallback_bytes = include_bytes!("./fallback.png");
        let fallback_texture = Texture::from_bytes(&context.device(), &context.queue(), fallback_bytes, "fallback.png")
            .expect("failed to load fallback image");

        fallback_texture
    }
}
