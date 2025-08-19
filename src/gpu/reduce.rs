use anyhow::Result;
use wgpu::util::DeviceExt;

pub struct ReduceGpu {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ReduceGpu {
    pub fn new(ctx: &crate::gpu::device::GpuContext) -> Self {
        let shader = ctx.load_shader("reduce.wgsl");
        let bind_group_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("reduce_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true}, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false}, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
            ],
        });

        let layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("reduce_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("reduce_pipeline"),
            layout: Some(&layout),
            module: &shader,
            entry_point: "main",
        });

        Self { pipeline, bind_group_layout }
    }

    pub fn reduce(&self, ctx: &crate::gpu::device::GpuContext, mut src_buffer: wgpu::Buffer, mut n: u32, encoder: &mut wgpu::CommandEncoder) -> wgpu::Buffer {
        let mut dst_buffer;
        while n > 1 {
            let groups = ((n + 511) / 512) as u32;
            dst_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("reduce_dst"),
                size: (groups * 4) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            let params_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("reduce_params"),
                contents: bytemuck::cast_slice(&[n,0,0,0]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

            let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("reduce_bind_group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: src_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: dst_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: params_buf.as_entire_binding() },
                ],
            });

            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("reduce_pass"),
                });
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(groups, 1, 1);
            }

            src_buffer = dst_buffer;
            n = groups;
        }
        src_buffer
    }
}