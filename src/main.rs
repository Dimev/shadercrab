pub mod drawer;

use glium::glutin;
use glutin::event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent};

use crate::drawer::*;

fn main() {
    // figure out what shader to load
    let (file_path, render_scale): (String, f32) = match &std::env::args().collect::<Vec<String>>()
        [..]
    {
        [_, x] => (x.clone(), 1.0),
        [_, x, p, y] if p == "-s" || p == "--scale" => (
            x.clone(),
            y.parse::<f32>().map_or_else(
                |_| {
                    println!("Could not parse scale as a float");
                    std::process::exit(0);
                },
                |x| x,
            ),
        ),
        _ => {
            // no valid arguments, show the help menu
            println!("Shadercrab {}", env!("CARGO_PKG_VERSION"));
            println!("A simple shadertoy emulator");
            println!("Usage:");
            println!("shadercrab [path] [-s|--scale render scale]");
            println!("	path: path to the shader file to use");
            println!("  render scale: what resolution to render at compared to window resolution");
            println!("");
            println!("This opens a window that shows the shader");
            println!("The shader is reloaded when the file is modified, or the r key is pressed");
            println!("Any shader errors are printed to the terminal");
            println!("");
            println!("Shader format:");
            println!("Shaders are in glsl, and need the function");
            println!("	mainImage(out vec4 fragColor, in vec2 fragCoord)");
            println!("where");
            println!("	fragColor: output color for the pixel, in sRGB color space");
            println!("	fragCoord: the pixel coordinate, with bottom left at (0, 0) and top right at (width, height)");
            println!("	           width and height are the width and height of the window");
            println!("");
            println!("The following constants are also defined:");
            println!("	float iTime: seconds since the shader was loaded");
            println!("	int iFrame: current frame number");
            println!("	vec3 iResolution: width, height and aspect ratio (y / x) of the window");
            println!(
                "	vec4 iMouse: xy: mouse position, changed when dragging with the left mouse button"
            );
            println!("	             zw: mouse button states (0 is up, 1 is down)");
            return;
        }
    };

    // time at which the file got modified
    // if it's not a valid file, show why we crashed
    let mut time_stamp = {
        match std::fs::metadata(&file_path) {
            Ok(x) => x
                .modified()
                .expect("Failed to access file access timestamp"),
            Err(reason) => {
                println!("Failed to open file: {:?}", reason);
                return;
            }
        }
    };

    // start up the event loop
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title(format!(
            "Shadercrab {} - {}",
            env!("CARGO_PKG_VERSION"),
            file_path
        ))
        .with_inner_size(glutin::dpi::PhysicalSize::new(800, 450));
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    println!(
        "GPU: {}\nVendor: {}\nOpenGL version: {}",
        display.get_opengl_renderer_string(),
        display.get_opengl_vendor_string(),
        display.get_opengl_version_string()
    );

    // current screen size
    let resolution = display.get_framebuffer_dimensions();

    // the actual drawing manager
    let mut drawer = Drawer::new(&display, resolution.0, resolution.1, render_scale);

	// load the program
    // mutable so we can reload later
    let program = load_program(&display, &file_path);
	
	// TODO PARSE
	drawer.main_image_program = program;

    // time since program start
    let mut start_time = std::time::Instant::now();

    // current frame
    let mut frame = 0;

    // mouse position and status
    let mut mouse_pos = (0, 0);
    let mut mouse_stat = (false, false);

    // whether we have focus
    let mut focus = false;

    // draw once to show it
    drawer.draw(&display, 0.0, frame, mouse_pos, mouse_stat, render_scale);

    // and run the event loop
    event_loop.run(move |event, _, control_flow| {
        // TODO reload program if the file changed
        // close if needed
        match event {
            Event::WindowEvent { event, .. } => match event {
                // close if requested
                WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit
                }
                // resized
                WindowEvent::Resized(s) => {
                    // rename the window to include the size
                    display.gl_window().window().set_title(&format!(
                        "Shadercrab {} - {}x{} - {}",
                        env!("CARGO_PKG_VERSION"),
                        (s.width as f32 * render_scale) as u32,
                        (s.height as f32 * render_scale) as u32,
                        file_path
                    ));
                }
                // check focus
                WindowEvent::Focused(f) => focus = f,
                // check mouse position
                WindowEvent::CursorMoved { position, .. } => {
                    mouse_pos = if mouse_stat.0 {
                        (position.x as u32, position.y as u32)
                    } else {
                        mouse_pos
                    }
                }
                // check mouse buttons
                WindowEvent::MouseInput {
                    state,
                    button: MouseButton::Left,
                    ..
                } => mouse_stat.0 = state == ElementState::Pressed,
                WindowEvent::MouseInput {
                    state,
                    button: MouseButton::Right,
                    ..
                } => mouse_stat.1 = state == ElementState::Pressed,
                // check if we need to reload
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.virtual_keycode == Some(VirtualKeyCode::R)
                        && input.state == ElementState::Released
                        && focus
                    {
                        println!("Reloaded shader");
                        drawer.main_image_program = load_program(&display, &file_path);
                        // reset the time as well
                        start_time = std::time::Instant::now();
                        // reset the frame
                        frame = 0;
                        // reset the mouse
                        mouse_pos = (0, 0);
                    }
                }
                _ => (),
            },
            Event::NewEvents(glutin::event::StartCause::ResumeTimeReached { .. })
            | Event::NewEvents(glutin::event::StartCause::Init) => {
                // check if the shader was edited
                let new_time_stamp = std::fs::metadata(&file_path).unwrap().modified().unwrap();
                if new_time_stamp != time_stamp {
                    // reload if it was
                    println!("Reloaded shader");
                    drawer.main_image_program = load_program(&display, &file_path);
                    // reset the time as well
                    start_time = std::time::Instant::now();
                    // reset the frame
                    frame = 0;
                    // reset the mouse
                    mouse_pos = (0, 0);
                    // and prevent continuous reloading
                    time_stamp = new_time_stamp;
                }

                // increment the frame
                frame += 1;

                // we're reached the end of the frame, redraw
                drawer.draw(
                    &display,
                    start_time.elapsed().as_secs_f32(),
                    frame,
                    mouse_pos,
                    mouse_stat,
                    render_scale,
                );
                // and request a redraw, at 60 fps
                *control_flow = glutin::event_loop::ControlFlow::WaitUntil(
                    std::time::Instant::now() + std::time::Duration::from_secs_f32(1.0 / 60.0),
                );
            }
            _ => (),
        }
    });
}
