#version 330 core
layout (location = 0) in vec3 aPos;


uniform mat4 view;
uniform mat4 projection;

out vec4 col;

void main() {
    gl_Position = projection * view * vec4(aPos, 1.0);
    col = vec4(0.0f, 1.0f, 0.0f, 1.0f);
}