
[[block]]
struct CameraUniform {
    view_matrix: mat4x4<f32>;
    aspect_ratio: f32;
};

[[group(0), binding(0)]]
var<uniform> camera: CameraUniform;

// The vertices only get stepped per instance.
//
// This allows saving some data from being sent multiple times if we had
// multiple vertices. Instead this is more of a VertexInput per glyph.
struct InstanceInput {
    [[builtin(vertex_index)]] vertex_index: u32;
    [[location(0)]] z: f32;
    [[location(1)]] left_top: vec2<f32>;
    [[location(2)]] right_bottom: vec2<f32>;
    [[location(3)]] texture_left_top: vec2<f32>;
    [[location(4)]] texture_right_bottom: vec2<f32>;
    [[location(5)]] color: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] texture_coords: vec2<f32>;
    [[location(1)]] font_color: vec4<f32>;
};

[[stage(vertex)]]
fn main(input: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    var position: vec2<f32> = vec2<f32>(0.0, 0.0);
    var left: f32 = input.left_top.x;
    var right: f32 = input.right_bottom.x;
    var top: f32 = input.left_top.y;
    var bottom: f32 = input.right_bottom.y;

    // Counter clockwise
    switch(i32(input.vertex_index)) {
        case 0: {
            position = vec2<f32>(left, top);
            out.texture_coords = input.texture_left_top;
        }
        case 1: {
            position = vec2<f32>(left, bottom);
            out.texture_coords = vec2<f32>(input.texture_left_top.x, input.texture_right_bottom.y);
        }
        case 2: {
            position = vec2<f32>(right, bottom);
            out.texture_coords = input.texture_right_bottom;
        }
        case 3: {
            position = vec2<f32>(right, top);
            out.texture_coords = vec2<f32>(input.texture_right_bottom.x, input.texture_left_top.y);
        }
    }

    out.position = vec4<f32>(position.x / 100.0 / 25.0, -position.y / 35.0 / 25.0, 0.0, 1.0);
    out.font_color = input.color;
    return out;
}

[[group(1), binding(0)]] var font_texture: texture_2d<f32>;
[[group(1), binding(1)]] var font_sampler: sampler;

[[stage(fragment)]]
fn main(input: VertexOutput) -> [[location(0)]] vec4<f32> {
    var alpha: f32 = textureSample(font_texture, font_sampler, input.texture_coords).r; // Single channel texture.

    if (alpha <= 0.0) {
        discard;
    }

    //return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    return input.font_color * vec4<f32>(1.0, 1.0, 1.0, alpha);
}
