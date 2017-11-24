#version 450

precision highp float;

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 tex_color;

layout(set = 0, binding = 0) uniform uni {
	mat4 proj;
} uniforms;

void main() {
	mat3 proj = mat3(uniforms.proj);
	vec2 pos = (proj * vec3(position, 1.0)).xy;
	gl_Position = vec4(pos, 0.0, 1.0);
	tex_color = color;
}
