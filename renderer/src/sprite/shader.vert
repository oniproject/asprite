#version 450

precision highp float;

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 color;
layout(location = 3) in uint texture;

layout(location = 0) out vec2 tex_coords;
layout(location = 1) out vec4 tex_color;
layout(location = 3) out uint tex_id;

layout(set = 0, binding = 0) uniform uni {
	mat4 proj;
} uniforms;

// Converts a color from linear light gamma to sRGB gamma
vec4 fromLinear(vec4 linearRGB) {
	bvec4 cutoff = lessThan(linearRGB, vec4(0.0031308));
	vec4 higher = vec4(1.055)*pow(linearRGB, vec4(1.0/2.4)) - vec4(0.055);
	vec4 lower = linearRGB * vec4(12.92);
	return mix(higher, lower, cutoff);
}

// Converts a color from sRGB gamma to linear light gamma
vec4 toLinear(vec4 sRGB) {
	bvec4 cutoff = lessThan(sRGB, vec4(0.04045));
	vec4 higher = pow((sRGB + vec4(0.055))/vec4(1.055), vec4(2.4));
	vec4 lower = sRGB/vec4(12.92);
	return mix(higher, lower, cutoff);
}

void main() {
	mat3 proj = mat3(uniforms.proj);
	vec2 pos = (proj * vec3(position, 1.0)).xy;
	gl_Position = vec4(pos, 0.0, 1.0);
	tex_coords = uv;

	tex_color = toLinear(color);
	tex_id = texture;
}
