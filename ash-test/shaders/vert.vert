#version 450

layout(set = 0, binding = 0) uniform WVPMatrices {
    mat4 world;
    mat4 view;
    mat4 projection;
} matrices;

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec3 inColor;
layout (location = 2) in vec2 inTexCoord;

layout(location = 0) out vec3 outColor;
layout(location = 1) out vec2 outTexCoord;

void main() {
    gl_Position = matrices.projection * matrices.view * matrices.world * vec4(inPosition, 0.0, 1.0);
    outColor = inColor;
    outTexCoord = inTexCoord;
}

