#version 300 es

in vec2 vert;
out vec2 tex_coord;
void main() {
    gl_Position = vec4(vert, 0.0, 1.0);
    tex_coord = (vert * vec2(1.0, -1.0) + 1.0)*0.5;
}
