//! Wgpu interface

use winit::window::Window;

/// Wgpu device and utilities
pub struct Graphics {
    instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    // draw to this
    surface: Option<wgpu::Surface>,
    // texture copy buffer
    // TODO

    // pipeline for copying to the framebuffer
}

impl Graphics {
    /// create a new device, with an internal framebuffer, if any
    fn new_with_maybe_framebuffer(window: Option<&Window>) -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        });

        let surface = window
            .map(|x| unsafe { instance.create_surface(x).ok() })
            .flatten();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: surface.as_ref(),
        }))?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                limits: wgpu::Limits::downlevel_webgl2_defaults(),
            },
            None,
        ))
        .ok()?;

        // TODO: pipeline for presenting

        Some(Graphics {
            instance,
            device,
            queue,
            surface,
        })
    }

    /// present the frame, if any
    pub fn present(&self) {
        todo!()
    }

    /// bind a texture to copy to the frame
    pub fn bind_texture(&mut self, texture: &wgpu::TextureView) {
        
    }
}
