#version 140

in vec2 _internal_vpos;
out vec4 _internal_fragcol;

uniform vec3 iResolution;
uniform vec4 iMouse;
uniform float iTime;

// TODO: insert shadertoy source here

void main() {
	_internal_fragcol = vec4(0.5, 0.5, 0.5, 1.0);
}