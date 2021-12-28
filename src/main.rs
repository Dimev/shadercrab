use glium::{glutin, Surface};
use glutin::event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent};

// load a program from a file path
fn load_program(display: &glium::Display, path: &str) -> Option<glium::program::Program> {
    // load the file
    let shader = match std::fs::read_to_string(path) {
        Ok(x) => x,
        Err(reason) => {
            println!("Failed to load shader: {:?}", reason);
            return None;
        }
    };

    // load build-in shaders
    let vertex_shader = include_str!("vertex.vert");

    // format the shader so it can go from shadertoy -> opengl
    let formatted_shader = format!(include_str!("fragment.frag"), shader);

    // make the shader input, because from_source does not give the ability to set srgb output
    let shader_input = glium::program::ProgramCreationInput::SourceCode {
        vertex_shader,
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: None,
        fragment_shader: &formatted_shader,
        transform_feedback_varyings: None,
        outputs_srgb: true,
        uses_point_size: false,
    };

    // make the program to run the shader
    // TODO ERROR REPORTING
    match glium::program::Program::new(display, shader_input) {
        Ok(x) => Some(x),
        Err(reason) => {
            let error = match reason {
                glium::program::ProgramCreationError::CompilationError(e, _) => e,
                x => format!("Error: {:?}", x),
            };

            println!("Failed to compile shader:\n{}", error);
            None
        }
    }
}

fn main() {
    // figure out what shader to load
    let file_path: String = match &std::env::args().collect::<Vec<String>>()[..] {
        [_, x] => x.clone(),
        _ => {
            // no valid arguments, show the help menu
            println!("Shadercrab {}", env!("CARGO_PKG_VERSION"));
            println!("A simple shadertoy emulator");
            println!("Usage:");
            println!("shadercrab [path]");
            println!("	path: path to the shader file to use");
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
    let wb = glutin::window::WindowBuilder::new().with_title(format!(
        "Shadercrab {} - {}",
        env!("CARGO_PKG_VERSION"),
        file_path
    ));
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    println!(
        "GPU: {}\nVendor: {}\nOpenGL version: {}",
        display.get_opengl_renderer_string(),
        display.get_opengl_vendor_string(),
        display.get_opengl_version_string()
    );

    // vertex buffer
    // not that important here as it's just a fullscreen quad
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vert {
            pos: [f32; 2],
        }

        glium::implement_vertex!(Vert, pos);

        glium::VertexBuffer::new(
            &display,
            &[
                Vert { pos: [-1.0, -1.0] },
                Vert { pos: [1.0, -1.0] },
                Vert { pos: [-1.0, 1.0] },
                Vert { pos: [1.0, 1.0] },
            ],
        )
        .unwrap()
    };

    let index_buffer = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &[0 as u8, 1, 2, 1, 2, 3],
    )
    .unwrap();

    // load the program
    // mutable so we can reload later
    let mut program = load_program(&display, &file_path);

    // draw loop
    let draw = move |display: &glium::Display,
                     program: Option<&glium::program::Program>,
                     time: f32,
                     frame: i32,
                     mouse_position: (u32, u32),
                     mouse_input: (bool, bool)| {
        // get the image size
        let resolution = display.get_framebuffer_dimensions();

        // make the uniforms and inputs
        let uniforms = glium::uniform! {
            iResolution: [resolution.0 as f32, resolution.1 as f32, resolution.1 as f32 / resolution.0 as f32],
            iFrame: frame as i32,
            iTime: time as f32,
            iMouse: [mouse_position.0 as f32, mouse_position.1 as f32, if mouse_input.0 { 1.0 } else { 0.0 }, if mouse_input.1 { 1.0 } else { 0.0 }]
        };

        // draw the frame
        let mut target = display.draw();
        target.clear_color_srgb(0.0, 0.0, 0.0, 0.0);

        // only draw if the program is valid
        if let Some(prog) = program {
            target
                .draw(
                    &vertex_buffer,
                    &index_buffer,
                    prog,
                    &uniforms,
                    &Default::default(),
                )
                .unwrap();
        }

        // and complete drawing
        target.finish().unwrap();
    };

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
    draw(
        &display,
        program.as_ref(),
        0.0,
        frame,
        mouse_pos,
        mouse_stat,
    );

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
                        s.width,
                        s.height,
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
                        program = load_program(&display, &file_path);
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
                    program = load_program(&display, &file_path);
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
                draw(
                    &display,
                    program.as_ref(),
                    start_time.elapsed().as_secs_f32(),
                    frame,
                    mouse_pos,
                    mouse_stat,
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
