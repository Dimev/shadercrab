#version 140

in vec2 pos;
out vec2 _internal_vpos;

void main() {
	// just set it to the position
	gl_Position = vec4(pos, 0.0, 1.0);
	_internal_vpos = pos;
}