#version 450

layout (set = 1, binding = 0) uniform sampler2D baseTexture;
layout (set = 1, binding = 1) uniform sampler2D aoTexture;
layout (set = 1, binding = 2) uniform sampler2D emissiveTexture;
 
layout (location = 0) in vec2 inTexCoord;

layout (location = 0) out vec4 outColor;

void main() 
{
    outColor = texture(baseTexture, inTexCoord) * 0.5 * texture(aoTexture, inTexCoord) + texture(emissiveTexture, inTexCoord);
}