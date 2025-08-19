// reduce.wgsl
// One pass of a staged reduction: in_data[0..n) -> out_data[0..num_groups)
// where num_groups = ceil(n / (WORKGROUP_SIZE * 2))

struct Params {
    n : u32,        // number of elements to reduce this pass
    _pad0 : u32,    // padding to 16-byte alignment for uniform buffers
    _pad1 : u32,
    _pad2 : u32,
};

@group(0) @binding(0) var<storage, read>        in_data  : array<f32>;
@group(0) @binding(1) var<storage, read_write>  out_data : array<f32>;
@group(0) @binding(2) var<uniform>              params   : Params;

// Tune to your GPU; 256 is a good portable default.
const WORKGROUP_SIZE : u32 = 256u;

var<workgroup> sdata : array<f32, WORKGROUP_SIZE>;

@compute @workgroup_size(WORKGROUP_SIZE, 1, 1)
fn main(@builtin(local_invocation_id)  lid : vec3<u32>,
        @builtin(global_invocation_id) gid : vec3<u32>,
        @builtin(workgroup_id)         wid : vec3<u32>) {

    // Each thread consumes 2 elements when possible for better bandwidth.
    let i0 = gid.x * 2u;
    let i1 = i0 + 1u;

    var sum : f32 = 0.0;
    if (i0 < params.n) { sum = in_data[i0]; }
    if (i1 < params.n) { sum = sum + in_data[i1]; }

    sdata[lid.x] = sum;
    workgroupBarrier();

    // Tree reduction in shared memory: halve active threads each step
    var stride = WORKGROUP_SIZE / 2u;
    loop {
        if (lid.x < stride) {
            sdata[lid.x] = sdata[lid.x] + sdata[lid.x + stride];
        }
        workgroupBarrier();
        if (stride == 1u) { break; }
        stride = stride / 2u;
    }

    // One partial sum per workgroup
    if (lid.x == 0u) {
        out_data[wid.x] = sdata[0u];
    }
}