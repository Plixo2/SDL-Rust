#version 330 core
layout (location = 0) in vec3 aPos;


uniform mat4 view;
uniform mat4 projection;
uniform vec4 color;
uniform vec2 position;

out vec2 pixelPos;

void main() {
    gl_Position = projection * view * vec4(aPos, 1.0);
    pixelPos = aPos.xy - position;
}