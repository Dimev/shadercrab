use clap::Parser;
use parse::Shadertoy;
use std::path::*;

mod parse;

/// Shadercrab, a dektop shadertoy emulator
#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
    /// what to multiply the resolution by when rendering
    #[clap(short, long, value_parser, default_value_t = 1.0)]
    scale: f32,

    /// file path to the config file or shader
    #[clap(value_parser)]
    config: PathBuf,

    /// treat the file as a shader
    #[clap(long, parse(from_flag))]
    shader: bool,
}

fn main() {
    // get the args
    let args = Arguments::parse();

    let toy = Shadertoy::new(&args.config, args.shader);

    println!("{:?}", toy);
}
