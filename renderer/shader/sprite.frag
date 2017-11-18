#version 450

layout(location = 0) in vec2 tex_coords;
layout(location = 1) in vec4 tex_color;
layout(location = 3) flat in uint tex_id;
layout(location = 0) out vec4 f_color;

layout (constant_id = 0) const uint TEXTURE_COUNT = 16;
layout(set = 1, binding = 0) uniform sampler2D tex[TEXTURE_COUNT];

void main() {
	f_color = texture(tex[tex_id], tex_coords) * tex_color;
}
