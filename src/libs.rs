use crate::compute_pipeline;
use cute::c;
use imagesize::size;
use rand::Rng;
use rayon::iter::IntoParallelIterator;
use std::time::{Duration, Instant};
use std::{iter, vec};
use std::{thread, time};
use wgpu::util::DeviceExt;
use winit::window::Window;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let (w, h) = match size("src/images/city.png") {
        Ok(dim) => (dim.width, dim.height),
        Err(why) => {
            println!("Error getting dimensions: {:?}", why);
            (0, 0)
        }
    };

    let logical_size = LogicalSize {
        width: w as u32,
        height: h as u32,
    };

    // let ten_millis = Duration::from_secs(1);
    // thread::sleep(ten_millis);

    window.set_inner_size(logical_size.to_physical::<u32>(0.5));
    let mut state = State::new(&window).await;
    state.resize(state.size);
    let mut print_time = Instant::now() + Duration::from_secs(1);
    let mut draws = 0;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                window.request_redraw();
                draws += 1;
                *control_flow = ControlFlow::Poll;
                if print_time < Instant::now() {
                    println!("{} Frames in 1 Second", draws);
                    print_time = Instant::now() + Duration::from_secs(1);
                    draws = 0
                }
            }

            Event::RedrawRequested(_) => {
                //state.update();
                match state.compute() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }

            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    });
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

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertices: Vec<Vertex>,
    vertex_buffer: Buffer,
    num_indices: u32,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,
    box_num: i32,
    num_triangles: i32,
    compute_pipeline: wgpu::ComputePipeline,
    binding: ComputeBinding,
}
//     reconstructed_texture: wgpu::Texture,

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let num_triangles = 2;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        // Texture Creation
        let diffuse_bytes = include_bytes!("images/city.png");
        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "images/city.png")
                .unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view), // CHANGED!
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler), // CHANGED!
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout], // NEW!
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        let mut vertices = State::get_vertices(num_triangles);
        let mut vert: &[Vertex] = &vertices[1..vertices.len()];

        let mut vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vert),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let u32_size = std::mem::size_of::<u32>() as u32;

        // let output_buffer_size = (u32_size * size * size) as wgpu::BufferAddress;
        // let output_buffer_desc = wgpu::BufferDescriptor {
        //     size: output_buffer_size,
        //     usage: wgpu::BufferUsages::COPY_DST
        //         // this tells wpgu that we want to read this buffer from the cpu
        //         | wgpu::BufferUsages::MAP_READ,
        //     label: None,
        //     mapped_at_creation: false,
        // };
        // let output_buffer = device.create_buffer(&output_buffer_desc);

        let num_indices = vert.len() as u32;
        let box_num = 1;
        let comp = compute_pipeline::ComputePipeline::get_pipeline(&device);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            compute_pipeline: comp.1,
            binding: comp.0,
            vertices,
            vertex_buffer,
            num_indices,
            diffuse_bind_group,
            diffuse_texture,
            box_num,
            num_triangles,
        }
    }

    pub fn get_vertices(num: i32) -> Vec<Vertex> {
        let mut rng = rand::thread_rng();
        let mut count = 0;
        let mut vertices: Vec<Vertex> = vec![];
        while count < num {
            let col = [
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..0.3),
            ];

            for _ in 1..3 {
                let p = [rng.gen_range(-1.3..1.3), rng.gen_range(-1.3..1.3), 0.0];
                let t = [(p[0] - 1.0) / -2.0, (p[1] - 1.0) / -2.0];

                vertices.push(Vertex {
                    position: p,
                    color: col,
                });
            }
            count = count + 1;
        }
        return vertices;
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        let mut d = State::get_vertices(self.num_triangles);
        self.vertices.append(&mut d);
        let vertices: &[Vertex] = &self.vertices[1..self.vertices.len()];

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        self.num_indices = vertices.len() as u32;
        self.vertex_buffer = vertex_buffer;
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw(0..self.num_indices, 0..1);

            //render_pass.set_vertex_buffer(0, self.vertex_buffer2.slice(..));
            //render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn compute(&mut self) -> Result<(), wgpu::SurfaceError> {
        let vertices = State::get_vertices(256);

        let brush = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Brush Buffer", m.name)),
            contents: bytemuck::cast_slice(&vertices),
            // UPDATED!
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Calculate the Stuff
        let binding = ComputeBinding {
            base_texture: self.binding.base_texture.view,
            base_texture_sampler: self.binding.base_texture.sampler,
            // reconstructed_texture: view,
            // reconstructed_texture_sampler: sampler,
            brush_shape: brush,
            compute_info: self.binding.compute_info,
        };
        let binder = binding.create_bind_group(&binding, device, Some("Error BindGroup"));
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Error Calc"),
        });

        // Compute Pass
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &binder, &[]);
            pass.dispatch(self.binding.compute_info.num_objects as u32, 1, 1);
        }
        queue.submit(iter::once(encoder.finish()));
        device.poll(wgpu::Maintain::Wait);

        Ok(())
    }
}

use crate::texture::Texture;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
//use web_sys::console::time;
use crate::texture;
use wgpu::{BindGroupLayout, Buffer, Instance, RenderPipeline, Sampler};
use winit::dpi::{LogicalSize, PhysicalSize};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32x4, // NEW!
                },
            ],
        }
    }
}

const NONE: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
