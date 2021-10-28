
use glyph_brush::{Extra, GlyphBrush, GlyphBrushBuilder, GlyphVertex, Section, Text, ab_glyph::FontArc};

use anyhow::Result;

use crate::renderer::{Texture, RenderContext, Vertex};

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

pub struct TextPass {
    shader: wgpu::ShaderModule,

    pipeline: wgpu::RenderPipeline,
    pipeline_layout: wgpu::PipelineLayout,
    brush: GlyphBrush<Vertex>,

    glyph_image: image::DynamicImage,
}

impl TextPass {
    fn new(context: &RenderContext) -> Result<Self> {
        let font = FontArc::try_from_slice(include_bytes!("./fonts/DejaVuSans.ttf"))?;
        let mut brush = GlyphBrushBuilder::using_font(font)
            .build::<Vertex, _>();

        let (width, height) = brush.texture_dimensions();
        //let glyph_texture = Texture::from_dimensions(context.device(), context.queue(), width, height, "glyph texture")?;
        let glyph_image = image::DynamicImage::new_rgba8(width, height);

        brush.queue(
    Section::default()
                .add_text(Text::new( "A quick brown fox jumps over the the lazy dog"))
        );

        let shader = context.device().create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Text Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

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
                label: Some("text_texture_bind_group_layout"),
            });

        let pipeline_layout =
            context.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Text Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &context.camera_bind_group_layout()],
                push_constant_ranges: &[],
            });

        let vertex_state = wgpu::VertexState {
            module: &shader,
            entry_point: "main",
            buffers: &[Vertex::desc()],
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
            label: Some("Text Render Pipeline"),
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

        Ok(Self {
            shader,
            pipeline,
            pipeline_layout,
            brush,
            glyph_image,
        })
    }

    pub fn update(&mut self, context: &RenderContext) {
        self.brush.process_queued(
            |rect, texture_data| {
                glyph_image.
            },
        )
    }
}

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

// Mostly just taken from the example of using gpu_glyph.
pub fn to_vertex(
    GlyphVertex {
        mut tex_coords,
        pixel_coords,
        bounds,
        extra,
    }: GlyphVertex,
) -> GlyphInstance {

    // handle overlapping bounds, modify uv_rect to preserve texture aspect
    if pixel_coords.max.x > bounds.max.x {
        let old_width = pixel_coords.width();
        pixel_coords.max.x = bounds.max.x;
        tex_coords.max.x = tex_coords.min.x + tex_coords.width() * pixel_coords.width() / old_width;
    }

    if pixel_coords.min.x < bounds.min.x {
        let old_width = pixel_coords.width();
        pixel_coords.min.x = bounds.min.x;
        tex_coords.min.x = tex_coords.max.x - tex_coords.width() * pixel_coords.width() / old_width;
    }

    if pixel_coords.max.y > bounds.max.y {
        let old_height = pixel_coords.height();
        pixel_coords.max.y = bounds.max.y;
        tex_coords.max.y = tex_coords.min.y + tex_coords.height() * pixel_coords.height() / old_height;
    }

    if pixel_coords.min.y < bounds.min.y {
        let old_height = pixel_coords.height();
        pixel_coords.min.y = bounds.min.y;
        tex_coords.min.y = tex_coords.max.y - tex_coords.height() * pixel_coords.height() / old_height;
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