// what to pass to the fragment shader
struct VertOut {
	@builtin(position) pos: vec4<f32>,
}

@vertex
fn main(@builtin(vertex_index) idx: u32) -> VertOut {

	// vertices
	var vert: vec2<f32> = vec2<f32>(-1.0, -1.0);

	// make triange
	// I hate wgsl
	if idx == 1u { vert = vec2<f32>(-1.0, 3.0); }
	if idx == 2u { vert = vec2<f32>(3.0, -1.0); }

	// triangle to cover the full screen
	var out: VertOut;
	out.pos = vec4<f32>(vert.x, vert.y, 0.0, 1.0);
	return out;
}