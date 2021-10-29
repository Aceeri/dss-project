use glyph_brush::{
    ab_glyph::FontArc, BrushAction, BrushError, Extra, GlyphBrush, GlyphBrushBuilder, GlyphVertex,
    Section, Text,
};

use anyhow::Result;

use wgpu::{
    BindGroup,
    BindGroupLayout,
    BindGroupEntry,
    BindingResource,
    util::DeviceExt,
};

use crate::renderer::{RenderContext, Texture, Vertex};

use std::mem;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct GlyphInstance {
    z: f32,
    left_top: [f32; 2],
    right_bottom: [f32; 2],
    texture_left_top: [f32; 2],
    texture_right_bottom: [f32; 2],
    color: [f32; 4],
}

impl GlyphInstance {
    pub const INITIAL_AMOUNT: u64 = 100;

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<GlyphInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute { // z
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute { // left top
                    offset: mem::size_of::<[f32; 1]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute { // right bottom
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute { // texture left top
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute { // texture right bottom
                    offset: mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute { // color
                    offset: mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }

    pub fn from_vertex(
        GlyphVertex {
            mut tex_coords,
            mut pixel_coords,
            bounds,
            extra,
        }: GlyphVertex,
    ) -> GlyphInstance {
        // Mostly just taken from the example of using gpu_glyph.

        println!("tex: {:?}, pix: {:?}", tex_coords, pixel_coords);

        // handle overlapping bounds, modify uv_rect to preserve texture aspect
        if pixel_coords.max.x > bounds.max.x {
            let old_width = pixel_coords.width();
            pixel_coords.max.x = bounds.max.x;
            tex_coords.max.x =
                tex_coords.min.x + tex_coords.width() * pixel_coords.width() / old_width;
        }

        if pixel_coords.min.x < bounds.min.x {
            let old_width = pixel_coords.width();
            pixel_coords.min.x = bounds.min.x;
            tex_coords.min.x =
                tex_coords.max.x - tex_coords.width() * pixel_coords.width() / old_width;
        }

        if pixel_coords.max.y > bounds.max.y {
            let old_height = pixel_coords.height();
            pixel_coords.max.y = bounds.max.y;
            tex_coords.max.y =
                tex_coords.min.y + tex_coords.height() * pixel_coords.height() / old_height;
        }

        if pixel_coords.min.y < bounds.min.y {
            let old_height = pixel_coords.height();
            pixel_coords.min.y = bounds.min.y;
            tex_coords.min.y =
                tex_coords.max.y - tex_coords.height() * pixel_coords.height() / old_height;
        }

        GlyphInstance {
            z: extra.z,
            left_top: [pixel_coords.min.x, pixel_coords.max.y],
            right_bottom: [pixel_coords.max.x, pixel_coords.min.y],
            texture_left_top: [tex_coords.min.x, tex_coords.max.y],
            texture_right_bottom: [tex_coords.max.x, tex_coords.min.y],
            color: extra.color,
        }
    }
}

pub struct TextPass {
    pipeline: wgpu::RenderPipeline,
    brush: GlyphBrush<GlyphInstance>,
    instances: wgpu::Buffer,
    supported_instances: u64,
    current_instances: usize,

    glyph_bind_group_layout: BindGroupLayout,
    glyph_texture: Texture,
    glyph_bind_group: wgpu::BindGroup,
}

impl TextPass {
    pub fn new(context: &RenderContext) -> Result<Self> {
        let font = FontArc::try_from_slice(include_bytes!("./fonts/Urbanist/Urbanist-Light.otf"))?;
        let brush = GlyphBrushBuilder::using_font(font).build();

        let glyph_bind_group_layout =
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
                    label: Some("TextPass::glyph_bind_group_layout"),
                });

        let (width, height) = brush.texture_dimensions();
        let (glyph_texture, glyph_bind_group) = Self::new_glyph_cache(context, &glyph_bind_group_layout, width, height)?;

        let shader = context
            .device()
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some("TextPass::shader.wgsl"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });


        let pipeline_layout =
            context
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("TextPass::pipeline_layout"),
                    bind_group_layouts: &[
                        &context.camera_bind_group_layout(),
                        &glyph_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let vertex_state = wgpu::VertexState {
            module: &shader,
            entry_point: "main",
            buffers: &[GlyphInstance::desc()],
        };

        let fragment_state = wgpu::FragmentState {
            module: &shader,
            entry_point: "main",
            targets: &[wgpu::ColorTargetState {
                format: context.config().format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent::OVER,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        };

        let primitive_state = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
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
                label: Some("TextPass::pipeline"),
                layout: Some(&pipeline_layout),
                vertex: vertex_state,
                fragment: Some(fragment_state),
                primitive: primitive_state,
                depth_stencil: Some(depth_stencil_state),
                multisample: wgpu::MultisampleState::default(),
            });

        let supported_instances = GlyphInstance::INITIAL_AMOUNT;
        let instances = context.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("TextPass::instances"),
            size: mem::size_of::<GlyphInstance>() as u64 * supported_instances,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            pipeline,
            brush,
            glyph_texture,
            glyph_bind_group,
            glyph_bind_group_layout,
            instances,
            supported_instances,
            current_instances: 0,
        })
    }

    pub fn update(&mut self, context: &RenderContext) -> Result<()> {
        self.process_queue(context)
    }

    pub fn process_queue(&mut self, context: &RenderContext) -> Result<()> {
        let Self {
            brush,
            glyph_texture,
            ..
        } = self;

        let brush_action = brush.process_queued(
            |rect, texture_data| {
                let offset = [rect.min[0], rect.min[1]];
                let size = [rect.width(), rect.height()];

                eprintln!("write {:?}, size {:?}", offset, size);
                glyph_texture.write_texture(context.queue(), offset, size, texture_data);
            },
            GlyphInstance::from_vertex,
        );

        match brush_action {
            Ok(BrushAction::Draw(instances)) => {
                if instances.len() as u64 > self.supported_instances {
                    eprintln!("expanding buffer");
                    context
                        .device()
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("TextPass::instances"),
                            contents: bytemuck::cast_slice(instances.as_slice()),
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        });
                } else {
                    eprintln!("writing to buffer");
                    context
                        .queue()
                        .write_buffer(
                            &self.instances,
                            0,
                            bytemuck::cast_slice(instances.as_slice()),
                        );

                }

                self.current_instances = instances.len();
            }
            Ok(BrushAction::ReDraw) => {},
            Err(BrushError::TextureTooSmall { suggested, .. }) => {
                let (width, height) = suggested;
                println!("resizing glyph texture to {}x{}", width, height);

                let (glyph_texture, glyph_bind_group) = Self::new_glyph_cache(context, &self.glyph_bind_group_layout, width, height)?;
                self.glyph_texture = glyph_texture;
                self.glyph_bind_group = glyph_bind_group;

                // This will re-queue draws for all glyphs so we don't have to worry
                // about writing the previous texture to the new texture.
                self.brush.resize_texture(width, height);

                // Down the rabbit hole weeeee
                self.process_queue(context)?;
            }
        }

        Ok(())
    }

    pub fn render(
        &mut self,
        context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let font_size = 72.0;
        let scale = (font_size * context.scale_factor() as f32).round();
        self.brush.queue(
            Section::default()
                .add_text(Text::new("a quick brown fox jumps over the lazy dog").with_color([1.0, 1.0, 1.0, 1.0]).with_scale(scale))
                .add_text(Text::new("A QUICK BROWN FOX JUMPS OVER THE LAZY DOG").with_color([1.0, 1.0, 1.0, 1.0]).with_scale(scale))
        );

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Sprite Pass"),
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
        render_pass.set_bind_group(0, &context.camera_bind_group(), &[]);
        render_pass.set_bind_group(1, &self.glyph_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.instances.slice(..));
        render_pass.draw(0..4, 0..self.current_instances as u32);
    }

    pub fn new_glyph_cache(context: &RenderContext, layout: &BindGroupLayout, width: u32, height: u32) -> Result<(Texture, BindGroup)> {
        eprintln!("new glyph cache, {}x{}", width, height);
        let texture = Texture::create_single_channel(
            context.device(),
            context.queue(),
            width,
            height,
            Some("TextPass::glyph_texture"),
        )?;

        let bind_group = context.device().create_bind_group(&wgpu::BindGroupDescriptor{
            layout: layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: BindingResource::TextureView(&texture.view), },
                BindGroupEntry { binding: 1, resource: BindingResource::Sampler(&texture.sampler), },
            ],
            label: Some("TextPass::glyph_texture_bind_group"),
        });

        Ok((texture, bind_group))
    }
}
