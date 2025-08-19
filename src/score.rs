use anyhow::Result;
use wgpu::util::DeviceExt;

pub struct GpuScorer {
    diff: crate::gpu::diff::DiffGpu,
    reduce: crate::gpu::reduce::ReduceGpu,
}

impl GpuScorer {
    pub fn new(ctx: &crate::gpu::device::GpuContext, w: u32, h: u32) -> Self {
        let diff = crate::gpu::diff::DiffGpu::new(ctx, w, h);
        let reduce = crate::gpu::reduce::ReduceGpu::new(ctx);
        Self { diff, reduce }
    }

    pub fn score(&self, ctx: &crate::gpu::device::GpuContext, orig: &wgpu::TextureView, cand: &wgpu::TextureView, w: u32, h: u32) -> Result<f32> {
        let size = (w * h * 4) as u64; // per-texel f32
        let diff_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("diff_buf"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("score_encoder"),
        });

        self.diff.compute_diff(ctx, orig, cand, &diff_buffer, w, h, &mut encoder);
        let final_buf = self.reduce.reduce(ctx, diff_buffer, w * h, &mut encoder);

        let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging_buf"),
            size: 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        encoder.copy_buffer_to_buffer(&final_buf, 0, &staging, 0, 4);

        ctx.queue.submit(Some(encoder.finish()));
        let buffer_slice = staging.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| ());
        ctx.device.poll(wgpu::Maintain::Wait);
        let data = buffer_slice.get_mapped_range()[..4].try_into().unwrap();
        let sum = f32::from_ne_bytes(data);
        Ok(sum)
    }
}