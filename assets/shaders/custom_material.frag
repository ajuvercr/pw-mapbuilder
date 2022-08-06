#version 450
layout(location = 0) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 1, binding = 0) uniform CustomMaterial {
    vec4 Color;
};

void main() {
    o_Target = Color;
}
