#version 460

// uniform buffer
layout(binding = 0) uniform ShadercrabInputs {{
	vec3 iResolution;
	float iTime;
}};

// vertex input

// output
layout(location = 0) out vec4 shadercrab_fragcol;

// textures 
{}

// common
{}

// shader
{}

// actual main
void main() {{

	// call shadertoy's mainImage
	mainImage(shadercrab_fragcol, vec2(0.0));

}}
