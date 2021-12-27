#version 140

in vec2 _internal_vpos;
out vec4 _internal_fragcol;

uniform vec3 iResolution;
uniform vec4 iMouse;
uniform float iTime;

// shadertoy source is inserted here
{}

// double braces to escape rust's formatter
void main() {{
	_internal_fragcol = vec4(0.2, 0.2, 0.2, 1.0);
}}