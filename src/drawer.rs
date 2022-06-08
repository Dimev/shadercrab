use glium::Surface;

use crate::buffer::*;

/// helper to actually draw the shaders
pub struct Drawer {
	// buffers that manage rendering
    pub buffers: [Buffer; 5],

	// where the buffers render to
	backbuffers: [glium::Texture2d; 5],

	// empty texture
    empty: glium::Texture2d,

	// main program to copy to the framebuffer
    main_program: glium::Program,

	// vertex buffer
    vertex_buffer: glium::VertexBuffer<Vert>,
    
	// size
	pub width: u32,
    pub height: u32,
}

impl Drawer {
    pub fn new(display: &glium::Display, width: u32, height: u32, scale: f32) -> Self {
        // scaled
        let width = (width as f32 * scale) as u32;
        let height = (height as f32 * scale) as u32;

        // vertex buffer
        // not that important here as it's just a fullscreen quad
        let vertex_buffer = {
            glium::VertexBuffer::new(
                display,
                &[
                    Vert { pos: [-1.0, -1.0] },
                    Vert { pos: [3.0, -1.0] },
                    Vert { pos: [-1.0, 3.0] },
                ],
            )
            .unwrap()
        };

        // program to copy from the render target to the output
        let main_program = glium::program::Program::new(
            display,
            glium::program::ProgramCreationInput::SourceCode {
                vertex_shader: "
				#version 140
	
				in vec2 pos;
				out vec2 vpos;
				
				void main() {
					// just set it to the position
					gl_Position = vec4(pos, 0.0, 1.0);
				
					// shadertoy's UV goes from (0, 0) to (1, 1), while gl's screen goes from (-1, -1) to (1, 1)
					vpos = pos * 0.5 + 0.5;
				}
				",
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                geometry_shader: None,
                fragment_shader: "
				#version 140

				in vec2 vpos;
				out vec4 fragcol;

				// where we get our image from
				uniform sampler2D main_image;

				void main() {

					// just copy it
					fragcol = texture(main_image, vpos);

				}
				",
                transform_feedback_varyings: None,
                outputs_srgb: true,
                uses_point_size: false,
            },
        )
        .unwrap();

        // empty texture
        let empty = glium::Texture2d::empty(display, 1, 1).unwrap();

        // buffers
		// TODO: PRETTYFY
		let (buf0, backbuf0) = Buffer::new(display, width, height, None, [Channel::Buffer(0), Channel::None, Channel::None, Channel::None]); 
		let (buf1, backbuf1) = Buffer::new(display, width, height, None, [Channel::None, Channel::None, Channel::None, Channel::None]); 
		let (buf2, backbuf2) = Buffer::new(display, width, height, None, [Channel::None, Channel::None, Channel::None, Channel::None]); 
		let (buf3, backbuf3) = Buffer::new(display, width, height, None, [Channel::None, Channel::None, Channel::None, Channel::None]); 
		let (buf4, backbuf4) = Buffer::new(display, width, height, None, [Channel::None, Channel::None, Channel::None, Channel::None]); 

		let buffers = [buf0, buf1, buf2, buf3, buf4];
		let backbuffers = [backbuf0, backbuf1, backbuf2, backbuf3, backbuf4];

        Self {
            empty,
            main_program,
            vertex_buffer,
            width,
            height,
            buffers,
			backbuffers,
        }
    }

    pub fn draw(
        &mut self,
        display: &glium::Display,
        time: f32,
        frame: i32,
        mouse_position: (u32, u32),
        mouse_input: (bool, bool),
        scale: f32,
    ) {
        // get the image size
        let resolution = display.get_framebuffer_dimensions();

		// resize if needed
        self.width = (resolution.0 as f32 * scale) as u32;
        self.height = (resolution.1 as f32 * scale) as u32;

        // draw buffers
        for (i, buffer) in self.buffers.iter_mut().enumerate() {
            buffer.draw(
                display,
				i,
                &mut self.backbuffers,
                time,
                frame,
                mouse_position,
                mouse_input,
                scale,
                &self.empty,
            );
        }

        // and draw to the main screen, grab the backbuffer as that's what's immediatly shown
        let mut target = display.draw();
        let uniform = glium::uniform! {
            main_image: self.backbuffers[0].sampled().wrap_function(glium::uniforms::SamplerWrapFunction::Clamp),
        };

        target
            .draw(
                &self.vertex_buffer,
                glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
                &self.main_program,
                &uniform,
                &Default::default(),
            )
            .unwrap();
        target.finish().unwrap();
    }
}
