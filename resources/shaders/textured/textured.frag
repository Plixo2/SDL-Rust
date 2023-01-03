#version 330 core

in vec2 uv;
out vec4 out_color;
uniform sampler2D image;
uniform vec4 color;

void main() {
    out_color = texture(image, uv) * color;
//    out_color = vec4(uv, 1.0 , 1.0);
}
