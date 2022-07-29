use naga::front::glsl::{Options, Parser};
use std::collections::BTreeMap;
use std::error::Error;

fn make_bindings(binding: usize, name: &str) -> String {
    // ideally this would use a proper AST to parse glsl, but that's too much work
    format!(
        "{}\n{}\n{}\n",
        // sampler, can't use sampler2D here because naga moment
        format!(
            "layout(set = {}, binding = {}) uniform sampler {}_shadercrab_internal_samp;",
            1,
            binding * 2,
            name
        ),
        // texture
        format!(
            "layout(set = {}, binding = {}) uniform texture2D {}_shadercrab_internal_tex;",
            1,
            binding * 2 + 1,
            name
        ),
        // define
        format!(
            "#define {} sampler2D({}_shadercrab_internal_tex, {}_shadercrab_internal_samp)",
            name, name, name
        )
    )
}

pub fn compile_shader(
    device: &wgpu::Device,
    shader: &str,
    common: &str,
    inputs: &BTreeMap<String, String>,
) -> Result<wgpu::ShaderModule, String> {
    // get the parser
    let mut parser = Parser::default();
    let options = Options::from(naga::ShaderStage::Fragment);

    // generate the right bindings
    let bindings = inputs
        .keys()
        .enumerate()
        .map(|(i, x)| make_bindings(i, x))
        .fold(String::new(), |acc, x| acc + &x);

    // make the shader
    let shader_code = format!(include_str!("template.glsl"), bindings, common, shader);

    // compile the shader
    // pretty print
    parser
        .parse(&options, &shader_code)
        .map(|x| {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Naga(x),
            })
        })
        .map_err(|x| {
            x.into_iter()
                .map(|naga_err| {
                    let mut err = naga_err.source();
                    let mut mesg = String::new();

                    while let Some(source) = err {
                        mesg = format!("{}\n{}", mesg, source);
                        err = Some(source);
                    }

                    mesg
                })
                .fold(String::new(), |acc, x| format!("{}\n{}", acc, x))
        })
}
