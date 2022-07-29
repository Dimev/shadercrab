use std::collections::HashMap;
use winit::window::Window;

/// uniforms representation
#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Unifomrs {
    /// vec3 iResolution;
    pub resolution: [f32; 4],

    /// float iTime;
    pub time: f32,

    /// float iTimeDelta;
    pub delta: f32,

    /// int	iFrame;
    pub frame: i32,

    /// float iFrameRate;
    pub framerate: f32,

    /// vec4 iMouse;
    pub mouse: [f32; 4],

    /// vec4 iDate;
    pub date: [f32; 4],
}

/// struct to represent a single channel render pipeline
pub struct ChannelRenderer {
    /// it's pipeline
    pub(crate) pipeline: wgpu::RenderPipeline,

    /// it's bind group
    pub(crate) bind_group: wgpu::BindGroup,

    /// it's inputs
    pub(crate) inputs: Vec<String>,
}

/// struct for the entire state of the renderer
pub struct Renderer {
    /// current width
    pub(crate) width: u32,

    /// current height
    pub(crate) height: u32,

    /// wgpu device
    pub(crate) device: wgpu::Device,

    /// queue
    pub(crate) queue: wgpu::Queue,

    /// surface to render to
    pub(crate) surface: wgpu::Surface,

    /// surface config
    pub(crate) config: wgpu::SurfaceConfiguration,

    /// copy to screen pipeline
    /// this handles outputting a texture to the window
    pub(crate) copy_to_screen_pipeline: wgpu::RenderPipeline,

    /// copy to screen sampler, to sample the texture
    pub(crate) copy_to_screen_sampler: wgpu::Sampler,

    /// bind group layout for the copy to screen
    pub(crate) copy_to_screen_bind_group_layout: wgpu::BindGroupLayout,

    /// copy to screen bindgroup, actual
    pub(crate) copy_to_screen_bind_group: wgpu::BindGroup,

    /// view of the dummy texture
    pub(crate) no_pipelines_texture_view: wgpu::TextureView,

    /// uniform buffer for uniforms
    pub(crate) uniforms: wgpu::Buffer,

    /// layout of the uniforms bind group
    pub(crate) uniforms_layout: wgpu::BindGroupLayout,

    /// uniform buffer bind group
    pub(crate) uniforms_bind_group: wgpu::BindGroup,

    // TODO: mipmapping pipeline
    /// all textures to use, and whether to resize them
    pub(crate) textures: HashMap<String, (bool, wgpu::Texture, wgpu::TextureView)>,

    /// all render pipelines
    pub(crate) pipelines: HashMap<String, ChannelRenderer>,
}

impl Renderer {
    /// render a frame to the window
    pub fn render(&mut self, width: u32, height: u32, uniforms: Unifomrs) {
        if width != self.width || height != self.height {
            self.config.width = width;
            self.config.height = height;
            self.width = width;
            self.height = height;

            // reconfigure because it changed
            self.surface.configure(&self.device, &self.config);

            // TODO: figure out how to resize the textures properly
        }

        // command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // update it
        self.queue
            .write_buffer(&self.uniforms, 0, bytemuck::cast_slice(&[uniforms]));

        // update buffers
        for (target, channel) in self.pipelines.iter() {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.textures[target].2,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            pass.set_pipeline(&channel.pipeline);
            pass.set_bind_group(0, &self.uniforms_bind_group, &[]);
            pass.set_bind_group(1, &channel.bind_group, &[]);

            // draw the triangle
            pass.draw(0..3, 0..1);
        }

        // render
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to get swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

        pass.set_pipeline(&self.copy_to_screen_pipeline);

        // bind our main texture, or none if there is none
        pass.set_bind_group(0, &self.copy_to_screen_bind_group, &[]);
        pass.draw(0..3, 0..1);

        std::mem::drop(pass);

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

                // this is needed to allow Rgba32Float textures to use shadertoy's style of sampling
                features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,

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
                label: Some("Copy To Screen"),
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
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
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
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let no_pipelines_texture_view =
            no_pipelines_texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Rgba32Float),
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
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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
                    min_binding_size: std::num::NonZeroU64::new(64),
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
            uniforms_layout,
            uniforms_bind_group,
            no_pipelines_texture_view,
            textures: HashMap::new(),
            pipelines: HashMap::new(),
        }
    }
}
