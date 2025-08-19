use anyhow::Result;
use wgpu::{Instance, Backends, Device, Queue, ShaderModule, Features};

pub struct GpuContext {
    pub device: Device,
    pub queue: Queue,
}

impl GpuContext {
    pub fn init() -> Result<Self> {
        let instance = Instance::new(Backends::PRIMARY);
        let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
            .ok_or_else(|| anyhow::anyhow!("No suitable GPU adapter found"))?;
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: Default::default(),
                required_limits: Default::default(),
            },
            None,
        ))?;
        Ok(Self { device, queue })
    }

    pub fn load_shader(&self, path: &str) -> ShaderModule {
        self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(path),
            source: wgpu::ShaderSource::Wgsl(include_str!(concat!("../assets/shaders/", path)).into()),
        })
    }
}