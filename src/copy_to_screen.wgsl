// texture to use
@group(0) @binding(0) var texture_in: texture_2d<f32>;
@group(0) @binding(1) var texture_sam: sampler;

// what to pass to the vertex shader
struct VertOut {
	@builtin(position) pos: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertOut {

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

@fragment
fn fs_main(vertex: VertOut) -> @location(0) vec4<f32> {

	// sample the input texture
        return textureSample(texture_in, texture_sam, vertex.pos.xy);
}
