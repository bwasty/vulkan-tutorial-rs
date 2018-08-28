#version 450
#extension GL_ARB_separate_shader_objects : enable

// NOTE: names must match the `Vertex` struct in Rust
layout(location = 0) in vec2 pos;
layout(location = 1) in vec3 color;

layout(location = 0) out vec3 fragColor;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
    fragColor = color;
}
