use crate::parse::Shadertoy;
use clap::Parser;
use std::path::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::renderer::Renderer;

mod parse;
mod renderer;

/// Shadercrab, a dektop shadertoy emulator
#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
    /// Scale at which to render
    /// 
    /// This affects the resolution of the textures to render to internally
    /// The new resolution is the window resolution * this scale factor
    #[clap(short, long, value_parser, default_value_t = 1.0)]
    scale: f32,

    /// file path to the config file or shader
    ///
    /// The file format used is toml.
    /// The field 'main' determines what shader will be rendered to the output screen
    /// The channels list allows determining what shaders render to what textures
    /// The common field describes the shader to include in all given shaders
    ///
    /// an example of a config
    /// ```toml
    /// 
    /// # render this to the window
    /// main = "main"
    ///
    /// # the common shader to include
    /// common = "common.glsl"
    ///
    /// # main pass, this is used as main
    /// [channels.main]
    /// shader = "main.glsl"
    /// inputs = { iChannel0 = "noise", iChannel1 = "bg" }
    ///
    /// # texture to input into the shader
    /// [channels.noise]
    /// texture = "noise.png"
    ///
    /// # and another channel
    /// [channels.bg]
    /// shader = "bg.glsl"
    /// ```
    #[clap(value_parser)]
    config: PathBuf,

    /// treat the file as a shader
    ///
    /// This means that the given file at the config is treated as the shader itself
    #[clap(long, parse(from_flag))]
    shader: bool,
}

fn main() {
    // get the args
    let args = Arguments::parse();

    let toy = match Shadertoy::new(&args.config, args.shader) {
        Ok(x) => x,
        Err(x) => {
            println!("Failed to parse config: {}", x);
            std::process::exit(1);
        }
    };

    // create a window
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).expect("Failed to make window");
    
    // start rendering
    window.request_redraw();

    // and a renderer
    let mut renderer = Renderer::new(&window);

    // start the event loop
    event_loop.run(move |event, _, control_flow| {
        // TODO: make this not consume CPU
        *control_flow = ControlFlow::Wait;

        // ckeck if the config changed
        if toy.check_reload() {
            println!("File changed, reconfigure");
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                // render!
                renderer.render(window.inner_size().width, window.inner_size().height);
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
