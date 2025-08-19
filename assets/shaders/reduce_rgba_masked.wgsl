// reduce_rgba_masked.wgsl
struct Params {
    n : u32,        // total number of pixels (w*h)
    width : u32,
    height : u32,
    _pad : u32,
};

@group(0) @binding(0) var img_tex  : texture_2d<f32>;
@group(0) @binding(1) var mask_tex : texture_2d<f32>;
@group(0) @binding(2) var<storage, read_write> out_partials : array<vec4<f32>>;
@group(0) @binding(3) var<uniform> params : Params;

const WORKGROUP_SIZE : u32 = 256u;
var<workgroup> sdata : array<vec4<f32>, WORKGROUP_SIZE>;

@compute @workgroup_size(WORKGROUP_SIZE, 1, 1)
fn main(@builtin(local_invocation_id)  lid : vec3<u32>,
        @builtin(global_invocation_id) gid : vec3<u32>,
        @builtin(workgroup_id)         wid : vec3<u32>) {

    // Flattened index over width*height
    let idx0 = gid.x * 2u;
    let idx1 = idx0 + 1u;

    // Helper to convert 1D -> 2D texel coordinates
    fn idx_to_xy(idx : u32, w : u32) -> vec2<i32> {
        let y = idx / w;
        let x = idx - y * w;
        return vec2<i32>(i32(x), i32(y));
    }

    var accum : vec4<f32> = vec4<f32>(0.0);

    if (idx0 < params.n) {
        let xy0 = idx_to_xy(idx0, params.width);
        let m0 = textureLoad(mask_tex, xy0, 0).x;
        if (m0 > 0.5) {
            let c0 = textureLoad(img_tex, xy0, 0).xyz;
            accum += vec4<f32>(c0, 1.0);
        }
    }

    if (idx1 < params.n) {
        let xy1 = idx_to_xy(idx1, params.width);
        let m1 = textureLoad(mask_tex, xy1, 0).x;
        if (m1 > 0.5) {
            let c1 = textureLoad(img_tex, xy1, 0).xyz;
            accum += vec4<f32>(c1, 1.0);
        }
    }

    sdata[lid.x] = accum;
    workgroupBarrier();

    var stride = WORKGROUP_SIZE / 2u;
    loop {
        if (lid.x < stride) {
            sdata[lid.x] = sdata[lid.x] + sdata[lid.x + stride];
        }
        workgroupBarrier();
        if (stride == 1u) { break; }
        stride = stride / 2u;
    }

    if (lid.x == 0u) {
        out_partials[wid.x] = sdata[0u];
    }
}