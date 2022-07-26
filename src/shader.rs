use crate::parse::{ShaderChannel, Shadertoy};
use naga::front::glsl::{Options, Parser};
use std::collections::HashMap;

fn format_shader(set: usize, binding: usize, name: &str) -> String {
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
    ) -> Result<HashMap<String, wgpu::ShaderModuleDescriptor>, HashMap<String, String>> {
        // result, to make it easier to add errors
        let mut shaders = Ok(HashMap::new());

        // naga parser
        let mut parser = Parser::default();

        // and we're doing fragment
        let options = Options::from(naga::ShaderStage::Fragment);

        // go over all shaders
        for (name, shader, inputs) in self.channels.iter().filter_map(|(name, x)| {
            if let ShaderChannel::Shader { shader, inputs } = x {
                Some((name, shader.clone(), inputs.clone()))
            } else {
                None
            }
        }) {
            // generate the list of inputs
            // this needs to change in case cubemaps/volumes get added
            let bindings = inputs
                .keys()
                .enumerate()
                .map(|(i, x)| format_shader(1, i, x))
                .fold(String::new(), |acc, x| acc + &x);

            // format the shader
            let shader_code = format!(include_str!("template.glsl"), bindings, self.common, shader);

            println!("{}", shader_code);

            // try to parse it
            let naga_shader_result = parser.parse(&options, &shader_code);

            // and add it
            // TODO: add it to the hashmap or error message set
            // TODO: better error messages
            if let Err(x) = naga_shader_result {
                for e in x {
                    println!(
                        "Error {}\n {}",
                        e.kind,
                        &shader_code[e.meta.to_range().unwrap()]
                    );
                }
            }
        }

        shaders
    }
}
