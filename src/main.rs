use glium::{glutin, Surface};

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

    // make the program to run the shader
    // TODO ERROR REPORTING
    match glium::program::Program::from_source(display, vertex_shader, &formatted_shader, None) {
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
    let file_path: String = match &std::env::args().collect::<Vec<String>>()[..]
    {
        [_, x] => x.clone(),
        _ => { 
			// no valid arguments, show the help menu
			println!("Invalid arguments");
			return;
		}
    };

	// time at which the file got modified
	// if it's not a valid file, show why we crashed
	let mut time_stamp = {
		match std::fs::metadata(&file_path) {
			Ok(x) => {
				x.modified().expect("Failed to access file access timestamp")
			},
			Err(reason) => {
				println!("Failed to open file: {:?}", reason);
				return;
			}
		}
	};

    // start up the event loop
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
		.with_title(format!("Shadercrab {} - {}", env!("CARGO_PKG_VERSION"), file_path));
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

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
    let draw = move |
		display: &glium::Display,			 
		program: Option<&glium::program::Program>,
                     time: f32,
                     mouse_position: (u32, u32),
                     mouse_input: (bool, bool)| {
        // get the image size
        let resolution = display.get_framebuffer_dimensions();

        // make the uniforms and inputs
        let uniforms = glium::uniform! {
            iResolution: [resolution.0 as f32, resolution.1 as f32, 1.0],
            iTime: time as f32,
            iMouse: [mouse_position.0 as f32, mouse_position.1 as f32, if mouse_input.0 { 1.0 } else { 0.0 }, if mouse_input.1 { 1.0 } else { 0.0 }]
        };

        // draw the frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);

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

	// mouse position and status
	let mut mouse_pos = (0, 0);
	let mut mouse_stat = (false, false);

    // draw once to show it
    draw(&display, program.as_ref(), 0.0, mouse_pos, mouse_stat);

    // and run the event loop
    event_loop.run(move |event, _, control_flow| {
        // TODO reload program if the file changed
        // close if needed
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                // close if requested
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit
                }
                _ => (),
            },
            glutin::event::Event::NewEvents(glutin::event::StartCause::ResumeTimeReached {
                ..
            })
            | glutin::event::Event::NewEvents(glutin::event::StartCause::Init) => {
                // check if the shader was edited
				let new_time_stamp = std::fs::metadata(&file_path).unwrap().modified().unwrap();
				if new_time_stamp != time_stamp {

					// reload if it was
					println!("Reloaded shader");
					program = load_program(&display, &file_path);
					time_stamp = new_time_stamp;
				}

                // we're reached the end of the frame, redraw
                draw(&display, program.as_ref(), start_time.elapsed().as_secs_f32(), (0, 0), (false, false));
                // and request a redraw
                *control_flow = glutin::event_loop::ControlFlow::WaitUntil(
                    std::time::Instant::now() + std::time::Duration::from_secs_f32(1.0 / 60.0),
                );
            },
			glutin::event::Event::DeviceEvent{ event, .. } => {
				match event {
					// if R was pressed, reload the shader and redraw, and reset the time as well
					glutin::event::DeviceEvent::Key(inp) => (
						if inp.virtual_keycode == Some(glutin::event::VirtualKeyCode::R) {
							println!("Reloaded shader");
							program = load_program(&display, &file_path);

							// reset the time as well
							start_time = std::time::Instant::now();
						}
					),
					_ => ()
				} 
			},
            _ => (),
        }
    });
}
