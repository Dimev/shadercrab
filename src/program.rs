// load a shader program
pub fn load_program(
    display: &glium::Display,
    shader: &str,
    common: &str,
) -> Option<glium::program::Program> {
    // load build-in shaders
    let vertex_shader = include_str!("vertex.vert");

    // format the shader so it can go from shadertoy -> opengl
    let formatted_shader = format!(include_str!("fragment.frag"), common, shader);

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
