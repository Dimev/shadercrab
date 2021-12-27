use glium::{glutin, Surface};

// load a program from a file path
fn load_program(display: &glium::Display, path: &str) -> Option<glium::program::Program> {
    // load build-in shaders
    let vertex_shader = include_str!("vertex.vert");
    let fragment_shader = include_str!("fragment.frag");

	// format the shader so it can go from shadertoy -> opengl

    // make the program to run the shader
    // TODO ERROR REPORTING
    match glium::program::Program::from_source(display, vertex_shader, fragment_shader, None) {
        Ok(x) => Some(x),
        Err(reason) => { println!("Failed to compile shader: {:?}", reason); None },
    }
}

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
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
    let mut program = load_program(&display, "fragment.frag");

    // draw loop
    let draw = move |program: Option<&glium::program::Program>,
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

    // draw once to show it
    draw(program.as_ref(), 0.0, (0, 0), (false, false));

    // and run the event loop
    event_loop.run(move |event, _, control_flow| {
        // by default, wait until the next frame
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(
            std::time::Instant::now() + std::time::Duration::from_secs_f32(1.0 / 60.0),
        );
        // TODO reload program if the file changed
        // close if needed
        match event {
            //			matc
            //			glutin::event::WindowEvent::CloseRequested => *control_flow = glutin::event_loop::ControlFlow::Exit,
            _ => (),
        }
    });
}
