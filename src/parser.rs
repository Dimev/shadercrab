use crate::drawer::*;
use crate::program::load_program;
use image::Rgba32FImage;
use std::path::{Path, PathBuf};
use toml::Value;

// TODO: nicer error reporting

/// what to give to the shader input
pub enum ShaderInput {
    Texture(Rgba32FImage),
    Buffer(usize),
    Keyboard,
    None,
}

impl Default for ShaderInput {
    fn default() -> Self {
        Self::None
    }
}

// full information for a parsed shader
#[derive(Default)]
pub struct Shadertoy {
    // file to use
    config_file: PathBuf,

    // files to watch, if these change, a reparse might be needed
    files_to_watch: Vec<PathBuf>,

    // main shader
    main_shader: String,

    // buffer shaders
    ichannel_shaders: [String; 4],

    // common shader
    common: String,

    // inputs for shaders
    main_inputs: [ShaderInput; 4],

    // inputs for the channels
    ichannel_inputs: [[ShaderInput; 4]; 4],
}

impl Shadertoy {
    // parses from a toml value
    fn from_toml(path: &Path, value: Value) -> Option<Self> {
        // parse our config
        let mut conf = Shadertoy::default();

        // set the file to watch
        conf.config_file = path.into();

        // TODO: modify these
        conf.files_to_watch = vec![path.into()];

        // get the blocks
        let main_shader = Self::toml_block(value.get("main")?, path)?;
        let channel_0 = Self::toml_block(value.get("ichannel0")?, path)?;
        let channel_1 = Self::toml_block(value.get("ichannel1")?, path)?;
        let channel_2 = Self::toml_block(value.get("ichannel2")?, path)?;
        let channel_3 = Self::toml_block(value.get("ichannel3")?, path)?;

        // special case
        conf.common = std::fs::read_to_string(value.get("common")?.as_str()?).ok()?;

        // and plug them into the conf
        conf.main_shader = main_shader.0;
        conf.main_inputs = main_shader.1;

        // and ichannels
        conf.ichannel_shaders = [channel_0.0, channel_1.0, channel_2.0, channel_3.0];
        conf.ichannel_inputs = [channel_0.1, channel_1.1, channel_2.1, channel_3.1];

        // and return it
        Some(conf)
    }

    // parses an ichannel (ichannel0 = ...) from a value
    fn toml_ichannel(value: &Value, channel: usize, path: &Path) -> Option<ShaderInput> {
        // try to get the right texture
        match value.get(format!("ichannel{}", channel)) {
            // just a buffer
            Some(Value::Integer(x)) => Some(ShaderInput::Buffer(*x as usize)),

            // read the actual texture file
            Some(Value::String(string)) => Some(ShaderInput::Texture(
                match image::io::Reader::open(path.join(&string)) {
                    Ok(x) => Some(x),
                    Err(x) => {
                        println!("Failed to load image {:?}: {:?}", path.join(&string), x);
                        None
                    }
                }?
                .decode()
                .ok()?
                .into_rgba32f(),
            )),
            _ => Some(ShaderInput::None),
        }
    }

    // parses a block (entire shader definition) from a toml value
    fn toml_block(value: &Value, path: &Path) -> Option<(String, [ShaderInput; 4])> {
        // get the shader
        let shader = match std::fs::read_to_string(path.join(value.get("shader")?.as_str()?)) {
            Ok(x) => Some(x),
            Err(x) => {
                println!(
                    "Failed to load shader {:?}: {:?}",
                    path.join(value.get("shader")?.as_str()?),
                    x
                );
                None
            }
        }?;
        // and inputs
        let inp_0 = Self::toml_ichannel(&value, 0, path)?;
        let inp_1 = Self::toml_ichannel(&value, 1, path)?;
        let inp_2 = Self::toml_ichannel(&value, 2, path)?;
        let inp_3 = Self::toml_ichannel(&value, 3, path)?;

        // and generate it
        Some((shader, [inp_0, inp_1, inp_2, inp_3]))
    }

    /// parse a config file from a given path
    pub fn new(path: &Path) -> Self {
        // load the contents
        let contents = match std::fs::read_to_string(path) {
            Ok(x) => x,
            Err(reason) => {
                println!("Failed to load shader {:?}: {:?}", path, reason);
                return Self {
                    files_to_watch: vec![path.into()],
                    config_file: path.into(),
                    ..Default::default()
                };
            }
        };

        // try and parse into a toml file
        match toml::from_str(&contents) {
            Ok(config) => match Self::from_toml(path, config) {
                Some(x) => x,
                _ => Self {
                    files_to_watch: vec![path.into()],
                    config_file: path.into(),
                    ..Default::default()
                },
            },
            Err(reason) => {
                // say the parse error reason
                println!("Failed to parse as toml: {}", reason);
                println!("Interpreting as shader instead");

                // return our
                Self {
                    files_to_watch: vec![path.into()],
                    config_file: path.into(),
                    main_shader: contents,
                    ..Default::default()
                }
            }
        }
    }
    /// optionally reloads the shader
    pub fn reload(&mut self) -> bool {
        false
    }

    /// apply this to a drawer
    pub fn load_shaders(&self, display: &glium::Display, drawer: &mut Drawer) {
        // load the main shader
        drawer.buffers[0].program = load_program(display, &self.common, &self.main_shader);

        // and the ichannel ones
        drawer.buffers[1].program = load_program(display, &self.common, &self.ichannel_shaders[0]);
        drawer.buffers[2].program = load_program(display, &self.common, &self.ichannel_shaders[1]);
        drawer.buffers[3].program = load_program(display, &self.common, &self.ichannel_shaders[2]);
        drawer.buffers[4].program = load_program(display, &self.common, &self.ichannel_shaders[3]);

        // and the inputs
        // drawer.buffers[0].channels
    }
}
