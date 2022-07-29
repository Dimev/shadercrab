#version 460

// uniform buffer
layout(binding = 0) uniform ShadercrabInternalInputs {{
	
	// shadertoy uniforms
	vec3 iResolution;	
	float iTime;
	float iTimeDelta;
	int	iFrame;	
	float iFrameRate;	
	vec4 iMouse;	
	vec4 iDate;
}};

// output
layout(location = 0) out vec4 shadercrab_internal_fragcol;

// textures 
{}

// common
{}

// shader
{}

// actual main
void main() {{

	// call shadertoy's mainImage
	mainImage(shadercrab_internal_fragcol, gl_FragCoord.xy * 0.5 + 0.5);

}}
