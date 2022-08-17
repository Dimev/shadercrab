struct VertexOut {
	@builtin(position) pos: vec4<f32>,
	@interpolate(linear) @location(0) uv: vec2<f32>,
}

@vertex
fn main(@builtin(vertex_index) idx: u32) -> VertexOut {

	// output
	var res: VertexOut;
	
	// vertices
	res.uv = vec2<f32>(-1.0, -1.0);

	// make triange
	// I hate wgsl
	if idx == 1u { res.uv = vec2<f32>(-1.0, 3.0); }
	if idx == 2u { res.uv = vec2<f32>(3.0, -1.0); }

	// triangle
	res.pos = vec4<f32>(res.uv.x, res.uv.y, 0.0, 1.0);

	// fix UV's
	res.uv *= vec2<f32>(0.5);
	res.uv += vec2<f32>(0.5);
	
	// triangle to cover the full screen
	return res;
}