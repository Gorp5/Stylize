// Vertex shader

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
    [[location(2)]] color: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var output_data: VertexOutput;
    output_data.tex_coords = model.tex_coords;
    output_data.color = model.color;
    output_data.clip_position = vec4<f32>(model.position, 1.0);
    return output_data;
}

// Fragment shader
[[group(0), binding(0)]]
var orig_texture: texture_2d<f32>;
[[group(0), binding(1)]]
var reconstructing_texture: texture_2d<f32>;
[[group(0), binding(2)]]
var sampler_type: sampler;

[[stage(fragment)]]
fn fs_main(orig_image_data: VertexOutput) -> [[location(0)]] vec4<f32> {
     let orig_color: vec4<f32> = textureSample(orig_texture, sampler_type, orig_image_data.tex_coords);
     let shape_color: vec4<f32> = orig_image_data.color;
     let alpha = shape_color[3] / 255.0;
     let transformed_color = vec4<f32>(
     orig_color[0] - (reconstructing_color[0] * (1.0 - alpha) + shape_color[0] * alpha),
     orig_color[1] - (reconstructing_color[1] * (1.0 - alpha) + shape_color[1] * alpha),
     orig_color[2] - (reconstructing_color[2] * (1.0 - alpha) + shape_color[2] * alpha),
     255.0);//(data[0] + color[0] * color[3], data[1] + color[1] * color[3], data[2] + color[2] * color[3], data[3] + color[3]);
     return transformed_color;
}