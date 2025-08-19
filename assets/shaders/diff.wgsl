// diff.wgsl
@group(0) @binding(0) var src_tex  : texture_2d<f32>;  // original
@group(0) @binding(1) var cand_tex : texture_2d<f32>;  // candidate/canvas
// out_diff length must be width*height; each entry is f32 (RGB L1)
@group(0) @binding(2) var<storage, read_write> out_diff : array<f32>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
    let dims = textureDimensions(src_tex);
    let w = dims.x;
    let h = dims.y;

    if (gid.x >= w || gid.y >= h) { return; }

    let xy = vec2<i32>(i32(gid.x), i32(gid.y));

    // Exact texel fetch (no sampler required). For UNORM formats this returns normalized floats.
    // See WGSL + WebGPU textureLoad semantics.
    let a = textureLoad(src_tex,  xy, 0); // RGBA
    let b = textureLoad(cand_tex, xy, 0);

    // L1 distance across RGB (ignore alpha; include if you want).
    let d = abs(a.xyz - b.xyz);
    let l1 = d.x + d.y + d.z;

    let idx = gid.y * w + gid.x;
    out_diff[idx] = l1;
}