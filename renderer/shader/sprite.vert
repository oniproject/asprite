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

void main() {
	mat3 proj = mat3(uniforms.proj);
	vec2 pos = (proj * vec3(position, 1.0)).xy;
	gl_Position = vec4(pos, 0.0, 1.0);
	tex_coords = uv;
	tex_color = color;
	tex_id = texture;
}
