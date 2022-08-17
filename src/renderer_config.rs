use image::DynamicImage;
use std::num::NonZeroU32;

use crate::parse::{ShaderChannel, Shadertoy};
use crate::renderer::{ChannelRenderer, Renderer};
use crate::shader::compile_shader;

impl Renderer {
    pub fn configure(&mut self, config: &Shadertoy) {
        // clear everything
        self.textures.clear();
        self.pipelines.clear();

        // make all textures for each channel
        for (name, channel) in config.channels.iter() {
            // figure out the size
            let (width, height) = match channel {
                ShaderChannel::Shader { .. } => (self.width, self.height),
                ShaderChannel::Image { image } => (image.width(), image.height()),
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
                format: wgpu::TextureFormat::Rgba32Float, // TODO: use a different one if this is not available
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
            });

            // write the data to the texture if available
            if let ShaderChannel::Image { image, .. } = channel {
                let image_dynamic = DynamicImage::from(image.to_rgba32f());

                // load the image
                self.queue.write_texture(
                    texture.as_image_copy(),
                    image_dynamic.as_bytes(),
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: NonZeroU32::new(image.width() * 4 * 4),
                        rows_per_image: NonZeroU32::new(image.height()),
                    },
                    wgpu::Extent3d {
                        width: image.width(),
                        height: image.height(),
                        depth_or_array_layers: 1,
                    },
                )
            }

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
        let vertex_shader = self
            .device
            .create_shader_module(wgpu::include_wgsl!("full_screen_triangle.wgsl"));

        // track shader compiler errors
        let mut failed_to_config = false;

        // make all bind groups for each shader channel
        // these are what the shader will take in
        // also make all render pipelines for each shader channel while we're at it
        for (name, shader, inputs) in config.channels.iter().filter_map(|(n, c)| match c {
            ShaderChannel::Shader { shader, inputs } => Some((n, shader, inputs)),
            _ => None,
        }) {
            // check if all channels exist
            if inputs
                .values()
                .map(|input| {
                    if !self.textures.contains_key(input) {
                        failed_to_config = true;
                        println!(
                            "Failed to config: input {} on {} doesn't exist",
                            input, name
                        );
                        true
                    } else {
                        false
                    }
                })
                .any(|x| x)
            {
                continue;
            }

            // compile the shader
            let fragment_shader =
                compile_shader(&self.device, name, shader, &config.common, &inputs);

            // if it failed, report the error
            let fragment_shader = match fragment_shader {
                Ok(x) => {
                    println!("compiling {} - V", name);
                    x
                }
                Err(x) => {
                    // report err
                    println!("compiling {} - X:\n{}", name, x);
                    failed_to_config = true;

                    // stop
                    continue;
                }
            };

            // make the bind group layout
            let layout_entries = (0..inputs.len() * 2)
                .into_iter()
                .map(|x| wgpu::BindGroupLayoutEntry {
                    binding: x as u32,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: if x & 1 != 0 {
                        wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        }
                    } else {
                        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
                    },
                    count: None,
                })
                .collect::<Vec<wgpu::BindGroupLayoutEntry>>();

            let layout = self
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &layout_entries,
                });

            // shared sampler for now
            let sampler = self
                .device
                .create_sampler(&wgpu::SamplerDescriptor::default());

            // and actual bind groups
            let entries = inputs
                .values()
                .enumerate()
                .flat_map(|(i, x)| {
                    // get the texture
                    // TODO: verify config
                    let texture = &self.textures[x].2;

                    // make the binding resource
                    let texture_res = wgpu::BindingResource::TextureView(&texture);

                    // get the sampler
                    // TODO!

                    // make the binding resource
                    let sampler_res = wgpu::BindingResource::Sampler(&sampler);

                    // make the iterator
                    [(i * 2, sampler_res), (i * 2 + 1, texture_res)]
                })
                .map(|(i, x)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: x,
                })
                .collect::<Vec<wgpu::BindGroupEntry>>();

            // make the bind group
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &entries,
            });

            // pipeline layout
            let pipeline_layout =
                self.device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[&self.uniforms_layout, &layout],
                        push_constant_ranges: &[],
                    });

            // and render pipeline
            let pipeline = self
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(name),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &vertex_shader,
                        entry_point: "main",
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &fragment_shader,
                        entry_point: "main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba32Float,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

            // and insert it
            self.pipelines.insert(
                name.clone(),
                ChannelRenderer {
                    pipeline,
                    bind_group,
                    inputs: inputs.values().cloned().collect(),
                },
            );
        }

        // if an error occurred, clear the pipelines and textures, as they aren't useful
        if failed_to_config {
            self.textures.clear();
            self.pipelines.clear();
        }

        // rebuild the copy to screen bind group
        self.copy_to_screen_bind_group =
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.copy_to_screen_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            // select the main output if available, otherwise the fallback
                            self.textures
                                .get(&config.main_shader)
                                .map(|x| &x.2)
                                .unwrap_or(&self.no_pipelines_texture_view),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.copy_to_screen_sampler),
                    },
                ],
            });
    }
}
