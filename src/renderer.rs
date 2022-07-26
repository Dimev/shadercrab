use crate::parse::{ShaderChannel, Shadertoy};
use std::collections::HashMap;
use winit::window::Window;

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

    /// view of the dummy texture
    no_pipelines_texture_view: wgpu::TextureView,

    /// uniform buffer for uniforms
    uniforms: wgpu::Buffer,

    /// uniform buffer bind group
    uniforms_bind_group: wgpu::BindGroup,

    // TODO: mipmapping pipeline
    /// all textures to use, and whether to resize them
    textures: HashMap<String, (bool, wgpu::Texture, wgpu::TextureView)>,

    /// all pipelines to use
    pipelines: HashMap<String, wgpu::RenderPipeline>,

    /// all bind groups to use
    bind_groups: HashMap<String, wgpu::BindGroup>,
}

impl Renderer {
    pub fn configure(&mut self, config: &Shadertoy) {
        // clear everything
        self.textures.clear();
        self.pipelines.clear();
        self.bind_groups.clear();

        // if it failed, print the message
        if false {
            // rebuild bind group to display the error texture
            self.copy_to_screen_bind_group =
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &self.copy_to_screen_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &self.no_pipelines_texture_view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.copy_to_screen_sampler),
                        },
                    ],
                });

            // stop early
            return;
        }

        // make all textures for each channel
        for (name, channel) in config.channels.iter() {
            // figure out the size
            let (width, height) = match channel {
                ShaderChannel::Shader { .. } => (self.width, self.height),
                ShaderChannel::Image { image } => image.dimensions(),
            };

            // make the texture and it's view
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
            });

            // view
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Rgba32Float),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });

            // whether to resize it
            let resize = match channel {
                ShaderChannel::Shader { .. } => true,
                _ => false,
            };

            // add it
            self.textures
                .insert(name.clone(), (resize, texture, texture_view));
        }

        // first, load the vertex shader for these
        let shader_module = self
            .device
            .create_shader_module(wgpu::include_wgsl!("full_screen_triangle.wgsl"));

        // make all bind groups for each shader channel
        // these are what the shader will take in
        // also make all render pipelines for each shader channel while we're at it
        for (name, shader, inputs) in config.channels.iter().filter_map(|(n, c)| match c {
            ShaderChannel::Shader { shader, inputs } => Some((n, shader, inputs)),
            _ => None,
        }) {

            // TODO: compile the shader
            // TODO: if it failed, pretty print the error

            // create the bind group layout for this shader

            // create the bind group

            // create the pipeline
        }
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

            // TODO: figure out how to resize the textures properly
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

        let info = adapter.get_info();
        println!("{} - {:?}", info.name, info.backend);

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
        let vertex_shader =
            device.create_shader_module(wgpu::include_wgsl!("full_screen_triangle.wgsl"));
        let fragment_shader =
            device.create_shader_module(wgpu::include_wgsl!("copy_to_screen.wgsl"));

        // actual pipline
        let copy_to_screen_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Copy to screen"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vertex_shader,
                    entry_point: "main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fragment_shader,
                    entry_point: "main",
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
            present_mode: wgpu::PresentMode::AutoVsync,
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

        // now the uniforms
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 1,
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        // it's bind group layout
        let uniforms_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(1),
                },
                count: None,
            }],
        });

        // and bind group
        let uniforms_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &uniforms_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
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
            uniforms,
            uniforms_bind_group,
            no_pipelines_texture,
            no_pipelines_texture_view,
            textures: HashMap::new(),
            pipelines: HashMap::new(),
            bind_groups: HashMap::new(),
        }
    }
}
