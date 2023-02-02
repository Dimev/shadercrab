//! Shader processing

use naga::{
    front::glsl::{Options, Parser},
    ShaderStage,
};

use crate::{graphics::Graphics, channel::ChannelDescriptor};

/// Compile a GLSL shader into a correct GLSL shader
pub fn apply_template(shader: &str, channels: &[ChannelDescriptor]) -> String {
    todo!()
}

/// Compile a GLSL shader to a wgpu ShaderModule
pub fn compile_shader(
    gfx: &Graphics,
    shader: &str,
    fragment: bool,
) -> Result<wgpu::ShaderModule, ()> {
    // compile the shader with naga
    let mut naga_parser = Parser::default();
    let options = Options::from(if fragment {
        ShaderStage::Fragment
    } else {
        ShaderStage::Vertex
    });

    // compile it and report errors
    let module = naga_parser.parse(&options, shader).map_err(|x| ())?;

    // make the shader source
    let shader_source = wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(module));

    // and shader
    Ok(gfx
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: shader_source,
        }))
}
