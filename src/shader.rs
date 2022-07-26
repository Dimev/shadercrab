use crate::parse::{ShaderChannel, Shadertoy};
use naga::front::glsl::{Options, Parser};
use std::collections::HashMap;

fn make_full_shader(set: usize, binding: usize, name: &str) -> String {
    // ideally this would use a proper AST to parse glsl, but that's too much work
    format!(
        "{}\n{}\n{}\n",
        // sampler, can't use sampler2D here because naga moment
        format!(
            "layout(set = {}, binding = {}) uniform sampler {}_samp;",
            set,
            binding * 2,
            name
        ),
        // texture
        format!(
            "layout(set = {}, binding = {}) uniform texture2D {}_tex;",
            set,
            binding * 2 + 1,
            name
        ),
        // define
        format!("#define {} sampler2D({}_tex, {}_samp)", name, name, name)
    )
}

impl Shadertoy {
    pub fn get_shaders(
        &self,
        device: &wgpu::Device,
    ) -> Result<HashMap<String, wgpu::ShaderModule>, HashMap<String, Vec<String>>> {
        // compiled shaders
        let mut shaders = HashMap::new();

        // found errors
        let mut errors = HashMap::new();

        // naga parser
        let mut parser = Parser::default();

        // and we're doing fragment
        let options = Options::from(naga::ShaderStage::Fragment);

        // go over all shaders
        for (name, shader, inputs) in self.channels.iter().filter_map(|(name, x)| match x {
            ShaderChannel::Shader { shader, inputs } => {
                Some((name, shader.clone(), inputs.clone()))
            }
            _ => None,
        }) {
            // generate the list of inputs
            // this needs to change in case cubemaps/volumes get added
            let bindings = inputs
                .keys()
                .enumerate()
                .map(|(i, x)| make_full_shader(1, i, x))
                .fold(String::new(), |acc, x| acc + &x);

            // format the shader
            let shader_code = format!(include_str!("template.glsl"), bindings, self.common, shader);

            // try to parse it
            let shader_module = parser
                .parse(&options, &shader_code)
                .map(|x| {
                    device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::ShaderSource::Naga(x),
                    })
                })
                .map_err(|x| {
                    x.iter()
                        .map(|y| format!("{:?}", y))
                        .collect::<Vec<String>>()
                });

            match shader_module {
                Ok(x) => {
                    shaders.insert(name.clone(), x);
                }
                Err(x) => {
                    errors.insert(name.clone(), x);
                }
            };
        }

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(shaders)
        }
    }
}
