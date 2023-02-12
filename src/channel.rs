//! shadertoy's iChannel equivalent

use std::{collections::BTreeMap, num::NonZeroU32};

use crate::{graphics::Graphics, uniforms::Uniforms};

/// A channel's texture format
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ChannelFormat {
    /// Cubemap
    Cubemap,

    /// 2D Image
    Image,

    /// Volume
    Volume,

    /// another channel, meaning this has to be resized
    Channel,
}

/// A channel texture
pub struct ChannelTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    format: ChannelFormat,
    sampler: wgpu::Sampler,
}

impl ChannelTexture {
    pub fn new(gfx: &Graphics, descriptor: &ChannelDescriptor, width: u32, height: u32) -> Self {
        // get the texture size
        let size = if descriptor.format == ChannelFormat::Channel {
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 0,
            }
        } else {
            wgpu::Extent3d {
                width: descriptor.width,
                height: descriptor.height,
                depth_or_array_layers: 0,
            }
        };

        // create the texture
        let texture = gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: size.max_mips(wgpu::TextureDimension::D2),
            sample_count: 0,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            view_formats: &[wgpu::TextureFormat::Rgba32Float],
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        // it's view
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            base_array_layer: 0,
            mip_level_count: NonZeroU32::new(size.max_mips(wgpu::TextureDimension::D2)),
            array_layer_count: None,
        });

        // and sampler
        let sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: size.max_mips(wgpu::TextureDimension::D2) as f32,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });

        Self {
            texture,
            view,
            sampler,
            format: descriptor.format,
        }
    }

    /// Get the bind group layout entry for the texture
    pub fn get_texture_layout(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                // TODO: filter based on if it's allowed
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }
    }

    /// Get the bind group layout entry for the texture sampler
    pub fn get_sampler_layout(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
            count: None,
        }
    }

    /// Get the bind group entry of the texture
    pub fn get_texture_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(&self.view),
        }
    }

    /// Get the bind group entry of the sampler
    pub fn get_sampler_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Sampler(&self.sampler),
        }
    }
}

/// A single channel
pub struct Channel {
    pipeline: wgpu::RenderPipeline,
    mipmap_pipeline: wgpu::RenderPipeline,
    texture_bind_group: wgpu::BindGroup,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    target: wgpu::Texture,
    target_view: wgpu::TextureView,
    textures: BTreeMap<String, ChannelTexture>,
}

/// Describes what will be in a channel
pub struct ChannelDescriptor<'a> {
    name: &'a str,
    format: ChannelFormat,
    width: u32,
    height: u32,
    rgba_f32_data: Option<&'a [u8]>,
}

impl Channel {
    pub fn new(
        gfx: &Graphics,
        shader: &str,
        channels: &[ChannelDescriptor],
        width: u32,
        height: u32,
    ) -> Self {
        // create all channels
        let mut textures = BTreeMap::new();

        for channel in channels {
            // create it
            textures.insert(
                channel.name.to_string(),
                ChannelTexture::new(gfx, channel, width, height),
            );
        }

        // create the bind groups
        let texture_bind_group_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &textures
                        .values()
                        .enumerate()
                        .map(|(idx, x)| {
                            [
                                x.get_texture_layout(idx as u32 * 2),
                                x.get_sampler_layout(idx as u32 * 2 + 1),
                            ]
                        })
                        .flatten()
                        .collect::<Vec<_>>(),
                });

        let texture_bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &texture_bind_group_layout,
            entries: &textures
                .values()
                .enumerate()
                .map(|(idx, x)| {
                    [
                        x.get_texture_entry(idx as u32 * 2),
                        x.get_sampler_entry(idx as u32 * 2 + 1),
                    ]
                })
                .flatten()
                .collect::<Vec<_>>(),
        });

        // uniform buffer and bind group
        // TODO!

        // create the shaders

        // create the pipeline
        let pipeline_layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = gfx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: todo!(),
                    entry_point: "main",
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: todo!(),
                    entry_point: "main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba32Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview: None,
            });

        // create the mipmapping pipeline
        // TODO!

        // return ourselves
        todo!()
    }

    pub fn draw(&mut self, gfx: &Graphics, encoder: &mut wgpu::CommandEncoder, uniforms: Uniforms) {
        // draw

        // mipmap

        todo!()
    }

    pub fn resize(
        &mut self,
        gfx: &Graphics,
        encoder: &mut wgpu::CommandEncoder,
        width: u32,
        height: u32,
    ) {
        // resize channel textures

        // rebind them

        // remake the target texture

        todo!()
    }

    pub fn copy_textures(
        &mut self,
        gfx: &Graphics,
        encoder: &mut wgpu::CommandEncoder,
        other: &Self,
        channel: &str,
    ) {
        todo!()
    }
}
