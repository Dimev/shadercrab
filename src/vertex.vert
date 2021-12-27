#version 140

in vec2 pos;
out vec2 _internal_vpos;

void main() {
	// just set it to the position
	gl_Position = vec4(pos, 0.0, 1.0);

	// shadertoy's UV goes from (0, 0) to (1, 1), while gl's screen goes from (-1, -1) to (1, 1)
	_internal_vpos = pos * 0.5 + 0.5;
}