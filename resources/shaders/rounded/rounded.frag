#version 330 core

uniform vec4 color;
uniform float roundness;
uniform vec4 outline_color;
uniform float outline;
uniform vec2 size;
uniform vec2 position;

out vec4 out_color;
in vec2 pixelPos;

const float smoothness = 1.1;

float udRoundBox( vec2 p, vec2 b, float r )
{
    return length(max(abs(p)-b+r,0.0))-r;
}

void main() {
   // out_color = color;
    //float maxX = size.x - roundness;
    //float maxY = size.y - roundness;
    //float alpha = 1;
    //if (pixelPos.x < roundness && pixelPos.y < roundness) {
    //    alpha *= 1.0 - smoothstep(roundness - smoothness, roundness + smoothness, length(pixelPos - vec2(roundness, roundness)));
    //} else if (pixelPos.x < roundness && pixelPos.y > maxY) {
    //    alpha *= 1.0 - smoothstep(roundness - smoothness, roundness + smoothness, length(pixelPos - vec2(roundness, maxY)));
    //} else if (pixelPos.x > maxX && pixelPos.y > maxY) {
    //    alpha *= 1.0 - smoothstep(roundness - smoothness, roundness + smoothness, length(pixelPos - vec2(maxX, maxY)));
    //} else if (pixelPos.x > maxX && pixelPos.y < roundness) {
    //    alpha *= 1.0 - smoothstep(roundness - smoothness, roundness + smoothness, length(pixelPos - vec2(maxX, roundness)));
    //} 
//
//
    //out_color = vec4(color.rgb,color.a * alpha);

vec2 half = size * 0.5;
if (roundness != 0)
{
    float dist = udRoundBox(pixelPos - half, half, roundness)+0.6;
    float smoothedAlpha =  1.0-smoothstep(0.0, smoothness ,dist);
    vec4 c = mix(vec4(color.rgb, smoothedAlpha * color.a), vec4(0.0), smoothstep(1.0, 0.0, smoothedAlpha));
    out_color = c;
} else {
    out_color = color;
}

{
    float fb = udRoundBox(pixelPos - half, half, roundness-outline);
    fb += outline * 0.5;
    float smoothedAlpha = smoothstep(1,0, abs(fb / (outline * 1.5)));
    vec4 c = mix(outline_color, out_color, smoothstep(1.0, 0.0, smoothedAlpha));
    out_color = c;
}
//out_color = vec4(0.0,1.0,0.0,1.0);

}
