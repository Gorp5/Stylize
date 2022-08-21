use crate::texture::Texture;
use anyhow::*;
use image::GenericImageView;
use rayon::prelude::*;
use std::ops::Range;
use std::path::Path;
//use web_sys::create_texture;
use wgpu::util::DeviceExt;
use wgpu::{TextureDescriptor, TextureDimension};

/*pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BrushVertex {
    position: [f32; 3],
    color: [f32; 4],
    score: f32,
    padding: [u32; 2],
}

impl Vertex for BrushVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<BrushVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    // format: wgpu::VertexFormat::Float32x3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    // format: wgpu::VertexFormat::Float32x3,
                    format: wgpu::VertexFormat::Float32x1,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ComputeInfo {
    num_objects: u32,
}

struct ComputeBinding {
    base_texture: wgpu::TextureView,
    base_texture_sampler: wgpu::Sampler,
    //reconstructed_texture: wgpu::TextureView,
    //reconstructed_texture_sampler: wgpu::Sampler,
    brush_shape: wgpu::Buffer,
    //texture_sampler: wgpu::Sampler,
    compute_info: ComputeInfo,
}

pub trait Bindable {
    fn layout_entries() -> Vec<wgpu::BindGroupLayoutEntry>;
    fn bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry>;
}

pub struct Binder<T: Bindable> {
    pub(crate) layout: wgpu::BindGroupLayout,
    _marker: std::marker::PhantomData<T>,
}

impl Bindable for ComputeBinding {
    fn layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            // Texture of the base image
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        // We will change the values in this buffer
                        read_only: false,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Sampler of the Texture of the base image
            // wgpu::BindGroupLayoutEntry {
            //     binding: 2,
            //     visibility: wgpu::ShaderStages::COMPUTE,
            //     ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            //     count: None,
            // },
            // wgpu::BindGroupLayoutEntry {
            //     binding: 3,
            //     visibility: wgpu::ShaderStages::COMPUTE,
            //     ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            //     count: None,
            // },
            // Dst Vertices
            // We'll store the computed tangent and bitangent here
            // wgpu::BindGroupLayoutEntry {
            //     binding: 1,
            //     visibility: wgpu::ShaderStages::COMPUTE,
            //
            //     ty: wgpu::BindingType::Buffer {
            //         ty: wgpu::BufferBindingType::Storage {
            //             // We will change the values in this buffer
            //             read_only: false,
            //         },
            //         has_dynamic_offset: false,
            //         min_binding_size: None,
            //     },
            //     count: None,
            // },

            // ComputeInfo
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ]
    }

    fn bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
        vec![
            // Src Vertices
            wgpu::BindGroupEntry {
                binding: 0,
                resource: self.base_texture.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: self.base_texture_sampler.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: self.brush_shape.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: self.compute_info.as_entire_binding(),
            },
        ]
    }
}

pub struct ComputePipeline {
    binding: ComputeBinding,
    pipeline: wgpu::ComputePipeline,
}
/*
impl ComputePipeline {
    pub fn get_pipeline(device: &wgpu::Device) -> Self {
        println!("Creating Compute pipeline");
        let shader_src = wgpu::include_wgsl!("compute_pipeline.comp.wgsl");
        let pipeline = create_compute_pipeline(
            device,
            &[&binder.layout],
            shader_src,
            Some("ComputePipeline"),
        );

        let diffuse_bytes = include_bytes!("images/city.png");
        let img = image::load_from_memory(diffuse_bytes)?;
        let base_texture = Texture::from_image(&device, &queue, &img, Some(&str));
        let dimensions = img.dimensions();
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let reconstructed_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("reconstruction"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        let view = reconstructed_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let compute_info = ComputeInfo {
            num_objects: vertices.len() as _,
            //num_indices: m.mesh.indices.len() as _,
        };

        let temp_brush = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} temp Buffer", m.name)),
            contents: bytemuck::cast_slice(&vertices),
            // UPDATED!
            usage: wgpu::BufferUsages::STORAGE,
        });

        let binding = ComputeBinding {
            base_texture: base_texture.view,
            base_texture_sampler: base_texture.sampler,
            // reconstructed_texture: view,
            // reconstructed_texture_sampler: sampler,
            brush_shape: temp_brush,
            compute_info,
        };

        Self { binding, pipeline }
    }
}

pub fn create_compute_pipeline(
    device: &wgpu::Device,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    shader_src: wgpu::ShaderModuleDescriptor,
    label: Option<&str>,
) -> wgpu::ComputePipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label,
        bind_group_layouts,
        push_constant_ranges: &[],
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label,
        layout: Some(&layout),
        module: &device.create_shader_module(&shader_src),
        entry_point: "main",
    });
    pipeline
}*/
*/
