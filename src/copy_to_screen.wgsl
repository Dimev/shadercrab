// texture to use
@group(0) @binding(0) var texture_in: texture_2d<f32>;
@group(0) @binding(1) var texture_sam: sampler;

// what to pass to the vertex shader
struct VertOut {
	@builtin(position) pos: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_index: u32) -> VertOut {
                
	// triangle to cover the full screen
        var x: f32 = -1.0;
	var y: f32 = -1.0;

	if in_index == 0u { x = 3.0; }
	if in_index != 0u { y = 3.0; }

	var out: VertOut;
	out.pos = vec4<f32>(x, y, 0.0, 1.0);
	return out;
}

@fragment
fn fs_main(vertex: VertOut) -> @location(0) vec4<f32> {

	// sample the input texture
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
