// texture to use
@group(0) @binding(0) var texture_in: texture_2d<f32>;
@group(0) @binding(1) var texture_sam: sampler;

@fragment
fn main(@builtin(position) vertex: vec4<f32>, @interpolate(linear) @location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {

	// sample the input texture
    return textureSample(texture_in, texture_sam, uv);
}
