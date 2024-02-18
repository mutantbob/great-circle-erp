
const vec2 verts[4] = vec2[4](
vec2(-2.0, -1.0),
vec2(1.0, -1.0),
vec2(-1.0, 1.0),
vec2(1.0, 1.0)
);
out vec2 tex_coord;
void main() {
    gl_Position = vec4(verts[gl_VertexID], 0.0, 1.0);
    tex_coord = (verts[gl_VertexID] * vec2(1.0,-1.0) + 1.0)*0.5;
}
