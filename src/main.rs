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
