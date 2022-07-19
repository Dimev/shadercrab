use crate::parse::Shadertoy;
use std::collections::HashMap;
use winit::window::Window;

/// struct to represent a texture channel
pub struct TextureChannel {
    /// texture
    texture: wgpu::Texture,
}

/// struct to represent a buffer
pub struct ShaderChannel {
    /// texture to render to
    texture: wgpu::Texture,

    /// shader to render with
    shader: wgpu::ShaderModule,

    /// render pipeline to use here
    pipeline: wgpu::RenderPipeline,

    /// layout for the bind group
    bind_group_layout: wgpu::BindGroupLayout,

    /// actual bind group
    bind_group: wgpu::BindGroup,
}

/// struct for the entire state of the renderer
pub struct Renderer {
    /// current width
    width: u32,

    /// current height
    height: u32,

    /// wgpu device
    device: wgpu::Device,

    /// queue
    queue: wgpu::Queue,

    /// surface to render to
    surface: wgpu::Surface,

    /// surface config
    config: wgpu::SurfaceConfiguration,

    /// copy to screen pipeline
    /// this handles outputting a texture to the window
    copy_to_screen_pipeline: wgpu::RenderPipeline,

    /// copy to screen sampler, to sample the texture
    copy_to_screen_sampler: wgpu::Sampler,

    /// bind group layout for the copy to screen
    copy_to_screen_bind_group_layout: wgpu::BindGroupLayout,

    /// copy to screen bindgroup, actual
    copy_to_screen_bind_group: wgpu::BindGroup,

    /// dummy texture to show when nothing is available
    no_pipelines_texture: wgpu::Texture,

    // TODO: uniform buffer for uniforms

    // TODO: uniform buffer layout

    // TODO: mipmapping pipeline
    /// texture channels
    texture_channels: HashMap<String, TextureChannel>,

    /// buffer channels, note that these also need to be updated
    /// as they render to a shader
    shader_channels: HashMap<String, ShaderChannel>,
}

impl Renderer {
    pub fn configure(&mut self, config: &Shadertoy) {
        todo!();
    }

    /// render a frame to the window
    pub fn render(&mut self, width: u32, height: u32) {
        if width != self.width || height != self.height {
            self.config.width = width;
            self.config.height = height;
            self.width = width;
            self.height = height;

            // reconfigure because it changed
            self.surface.configure(&self.device, &self.config);
        }

        // update buffers

        // render
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to get swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        renderpass.set_pipeline(&self.copy_to_screen_pipeline);

        // bind our main texture, or none if there is none
        renderpass.set_bind_group(0, &self.copy_to_screen_bind_group, &[]);

        renderpass.draw(0..3, 0..1);

        std::mem::drop(renderpass);

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    /// make a new renderer from a window
    pub fn new(window: &Window) -> Self {
        // window size
        let size = window.inner_size();

        // get the gpu
        let instance = wgpu::Instance::new(wgpu::Backends::all());

        // make the window
        let surface = unsafe { instance.create_surface(window) };

        // get the actual gpu
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .expect("Failed to get gpu");

        // device and queue
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),

                // we want to run on a variety of gpu's, so low level is good.
                // compute also isn't needed
                limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
            },
            None,
        ))
        .expect("Failed to get device");

        // get the pipeline to copy to screen
        // bind groups, just a single texture
        let copy_to_screen_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
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
            });

        // format we can render to
        let swapchain_format = surface.get_supported_formats(&adapter)[0];

        // layout first
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Copy to screen layout"),
            bind_group_layouts: &[&copy_to_screen_bind_group_layout],
            push_constant_ranges: &[],
        });

        // shader for the copy to screen
        let shader = device.create_shader_module(wgpu::include_wgsl!("copy_to_screen.wgsl"));

        // actual pipline
        let copy_to_screen_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Copy to screen"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        // texture sampler
        let copy_to_screen_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });

        // configure the surface
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        // dummy texture
        let no_pipelines_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let no_pipelines_texture_view =
            no_pipelines_texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Rgba8Unorm),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });

        // bind group to help the dummy pipeline
        let copy_to_screen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &copy_to_screen_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&no_pipelines_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&copy_to_screen_sampler),
                },
            ],
        });

        surface.configure(&device, &config);

        // now just return the struct
        Self {
            device,
            queue,
            surface,
            config,
            width: size.width,
            height: size.height,
            copy_to_screen_pipeline,
            copy_to_screen_sampler,
            copy_to_screen_bind_group_layout,
            copy_to_screen_bind_group,
            no_pipelines_texture,
            texture_channels: HashMap::new(),
            shader_channels: HashMap::new(),
        }
    }
}
