use anyhow::Result;
use wgpu::util::DeviceExt;
use wgpu::{Device, Queue, Texture, TextureDescriptor, TextureDimension, TextureFormat, Extent3d, TextureUsages};
use wgpu::util::TextureDataOrder;

/// Upload a CPU RGBA8 image buffer into a GPU Texture and return it with its `TextureView`.
pub fn upload_image_to_texture(
    device: &Device,
    queue: &Queue,
    data: &[u8],
    width: u32,
    height: u32,
) -> Result<Texture> {
    // Texture descriptor with RGBA8Unorm format (8-bit per channel).
    let texture_desc = TextureDescriptor {
        label: Some("upload_image_to_texture"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
        view_formats: (),
    };

    // Upload texture in one go.
    let texture = device.create_texture_with_data(
        queue,
        &texture_desc,
        TextureDataOrder::default(),
        data,
    );

    Ok(texture)
}

/// Create a GPU buffer (storage) with `count` floats initialized to `init_value`.
pub fn float_buffer_with_value(
    device: &Device,
    count: usize,
    init_value: f32,
    label: &str,
) -> wgpu::Buffer {
    // Pre-fill a Vec with the specified initial value.
    let init_data: Vec<f32> = vec![init_value; count];
    let byte_data = bytemuck::cast_slice(&init_data);

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: byte_data,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    })
}