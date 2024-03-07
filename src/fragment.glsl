#version 300 es

precision mediump float;
uniform sampler2D world;
uniform mat3 rotation;
out vec4 out_color;
in vec2 tex_coord;

#define PI 3.14159
#define TAU (2.0*PI)

vec2 frac_to_radians(float longitude_frac, float latitude_frac)
{
    float theta = longitude_frac * TAU;
    float phi = (latitude_frac - 0.5) * PI;
    return vec2(theta, phi);
}

vec3 spherical_to_cartesian(float theta, float phi)
{
    float z = sin(phi);
    float r = cos(phi);
    float x = -cos(theta) * r;
    float y = -sin(theta) * r;

    return vec3(x, y, z);
}

float my_fmod(float a, float b)
{
    return a - b*floor(a/b);
}

vec2 cartesian_to_lat_long(vec3 xyz)
{
    float r = length(xyz.xy);
    float phi = atan(xyz.z, r);
    float theta = atan(-xyz.y, -xyz.x);
    float v = phi / PI + 0.5;
    float u = theta / TAU;

    return vec2(my_fmod(u, 1.0), my_fmod(v, 1.0));
}

vec2 remap(vec2 src, mat3 matrix)
{
    vec2 theta_phi = frac_to_radians(src.x, src.y);

    vec3 xyz = spherical_to_cartesian(theta_phi.x, theta_phi.y);

    return cartesian_to_lat_long(matrix * xyz);
}

void main() {
    out_color = texture(world, remap(tex_coord, rotation));
}
