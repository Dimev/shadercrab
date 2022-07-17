#version 140

// internal inputs, from the fragment shader 
in vec2 _internal_vpos;
out vec4 _internal_fragcol;

// shadertoy inputs
uniform vec3 iResolution;
uniform vec4 iMouse;
uniform float iTime;
uniform int iFrame;

// textures
uniform sampler2D iChannel0;
uniform sampler2D iChannel1;
uniform sampler2D iChannel2;
uniform sampler2D iChannel3;


// shadertoy common inserted here, 
// with rust format strings
{}

// shadertoy source is inserted here
{}

// double braces to escape rust's formatter
void main() {{

	// shadertoy has mainImage, which takes in the fragcolor to output, and the UV (frag coordinate) multiplied by the resolution
	// we have these in the file, so we can just grab the function and render it
	mainImage(_internal_fragcol, _internal_vpos * iResolution.xy);

}}