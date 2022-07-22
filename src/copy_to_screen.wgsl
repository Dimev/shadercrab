// texture to use
@group(0) @binding(0) var texture_in: texture_2d<f32>;
@group(0) @binding(1) var texture_sam: sampler;

// what we got from the vertex shader
struct VertOut {
	@builtin(position) pos: vec4<f32>,
}

@fragment
fn main(vertex: VertOut) -> @location(0) vec4<f32> {

	// sample the input texture
    return textureSample(texture_in, texture_sam, vertex.pos.xy);
}
