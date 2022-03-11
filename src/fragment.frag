#version 140

in vec2 _internal_vpos;
out vec4 _internal_fragcol;

uniform vec3 iResolution;
uniform vec4 iMouse;
uniform float iTime;
uniform int iFrame;

uniform sampler2D iChannel0;
uniform sampler2D iChannel1;
uniform sampler2D iChannel2;
uniform sampler2D iChannel3;

// shadertoy source is inserted here
// this is just rust's format string
{}

// double braces to escape rust's formatter
void main() {{

	// shadertoy has mainImage, which takes in the fragcolor to output, and the UV (frag coordinate) multiplied by the resolution
	// we have these in the file, so we can just grab the function and render it
	mainImage(_internal_fragcol, _internal_vpos * iResolution.xy);

}}