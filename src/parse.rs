use image::Rgba32FImage;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::*;
use std::time::SystemTime;

/// helper function to check when a path changed
fn file_changed_time(path: &Path) -> Result<SystemTime, String> {
    std::fs::metadata(path)
        .map_err(|x| format!("File {} not found: {}", path.to_string_lossy(), x))?
        .modified()
        .map_err(|x| {
            format!(
                "Could not get modified timestamp of file {}: {}",
                path.to_string_lossy(),
                x
            )
        })
}

/// struct storing a single shader's config
#[derive(Debug, Deserialize)]
pub struct Channel {
    /// path to the shader file
    pub shader: Option<PathBuf>,

    /// path to the image file
    pub image: Option<PathBuf>,

    /// list of inputs
    pub inputs: Option<HashMap<String, String>>,
}

/// struct for a full shader definition
#[derive(Debug, Deserialize)]
pub struct Config {
    /// shader main input/output
    pub output: String,

    /// shader common
    pub common: Option<PathBuf>,

    /// shader defines
    pub channels: HashMap<String, Channel>,
}

/// single shader channel
#[derive(Debug)]
pub enum ShaderChannel {
    Shader {
        shader: String,
        inputs: HashMap<String, String>,
    },
    Image {
        image: Rgba32FImage,
    },
}

/// full shader descriptor
#[derive(Debug, Default)]
pub struct Shadertoy {
    /// all files to watch
    files: Vec<(PathBuf, SystemTime)>,

    /// config file itself
    config_file: PathBuf,

    /// which shader to show to the screen
    main_shader: String,

    /// whether this is a shader, or a full config
    is_shader: bool,

    /// common shader
    common: String,

    /// all channels
    channels: HashMap<String, ShaderChannel>,
}

impl Shadertoy {
    /// check if it needs to be reloaded
    pub fn check_reload(&self) -> bool {
        self.files.iter().any(|(file_path, time_changed)| {
            if let Ok(time) = file_changed_time(file_path) {
                // only reload if the changed time is newer than the old time
                time > *time_changed
            } else {
                // we need to reload if the file stopped existing
                true
            }
        })
    }

    /// make a new shadertoy, from the given config file path
    pub fn new(path: &Path, is_shader: bool) -> Result<Self, String> {
        if is_shader {
            // see when the file changed
            let file_changed = file_changed_time(path)?;

            // load the shader file
            let shader = std::fs::read_to_string(path)
                .map_err(|x| format!("File {} not found: {}", path.to_string_lossy(), x))?;

            // make the hashmap
            let mut channels = HashMap::new();
            channels.insert(
                "main".into(),
                ShaderChannel::Shader {
                    shader,
                    inputs: HashMap::new(),
                },
            );

            Ok(Self {
                files: vec![(path.into(), file_changed)],
                config_file: path.into(),
                main_shader: "main".into(),
                common: "".into(),
                is_shader,
                channels,
            })
        } else {
            // parse the file
            let config: Config = toml::from_str(
                &std::fs::read_to_string(path)
                    .map_err(|x| format!("File {} not found: {}", path.to_string_lossy(), x))?,
            )
            .map_err(|x| format!("Failed to parse {}: {}", path.to_string_lossy(), x))?;

            // get the full path of the directory
            let directory = path
                .parent()
                .ok_or(format!(
                    "Failed to get parent directory of {}",
                    path.to_string_lossy()
                ))?
                .canonicalize()
                .map_err(|x| {
                    format!(
                        "Failed to get full path of {}: {}",
                        path.to_string_lossy(),
                        x
                    )
                })?;

            // try and load the common shader
            let common = if let Some(common) = &config.common {
                std::fs::read_to_string(if common.is_absolute() {
                    common.clone()
                } else {
                    directory.join(common)
                })
                .map_err(|x| {
                    format!(
                        "File {} not found: {}",
                        (if common.is_absolute() {
                            common.clone()
                        } else {
                            directory.join(common)
                        })
                        .to_string_lossy(),
                        x
                    )
                })?
            } else {
                String::new()
            };

            // setup without looping
            let mut toy = Self {
                files: vec![(path.into(), file_changed_time(path)?)],
                config_file: path.into(),
                main_shader: config.output,
                common,
                ..Default::default()
            };

            // go over all channels in the shader, and convert them
            for (name, channel) in config.channels.into_iter() {
                let shader_channel = match (channel.shader, channel.image) {
                    (Some(shader_file), None) => {
                        // load the shader, and it's file
                        let shader_path = if shader_file.is_absolute() {
                            shader_file
                        } else {
                            directory.join(shader_file)
                        };
                        let shader = std::fs::read_to_string(&shader_path).map_err(|x| {
                            format!("Failed to load {}: {}", shader_path.to_string_lossy(), x)
                        })?;

                        toy.files
                            .push((shader_path.clone(), file_changed_time(&shader_path)?));

                        // then construct the channel
                        ShaderChannel::Shader {
                            shader,
                            inputs: channel.inputs.unwrap_or(HashMap::new()),
                        }
                    }
                    (None, Some(image_file)) => {
                        // load the shader, and it's file
                        let image_path = if image_file.is_absolute() {
                            image_file.clone()
                        } else {
                            directory.join(image_file)
                        };
                        let image = image::open(&image_path).map_err(|x| {
                            format!("Failed to load {}: {}", image_path.to_string_lossy(), x)
                        })?;

                        toy.files
                            .push((image_path.clone(), file_changed_time(&image_path)?));

                        // then construct the channel
                        ShaderChannel::Image {
                            image: image.to_rgba32f(),
                        }
                    }
                    _ => {
                        return Err(format!(
                            "Channel {} had both a shader and image, which isn't allowed",
                            name
                        ))
                    }
                };

                // add it to the toy
                toy.channels.insert(name, shader_channel);
            }

            Ok(toy)
        }
    }
}
