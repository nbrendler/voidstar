in vec2 co;
in vec2 tex_co;
in vec2 position;

out vec2 v_tex_co;

// use these once this bug is fixed:
// https://github.com/phaazon/luminance-rs/issues/434
uniform mat4 model;
uniform mat4 projection;
uniform mat4 view;
uniform vec4 mc0;
uniform vec4 mc1;
uniform vec4 mc2;
uniform vec4 mc3;
uniform vec4 pc0;
uniform vec4 pc1;
uniform vec4 pc2;
uniform vec4 pc3;
uniform vec4 vc0;
uniform vec4 vc1;
uniform vec4 vc2;
uniform vec4 vc3;
uniform vec2 world_bounds;

void main()
{
    mat4 m = mat4(mc0, mc1, mc2, mc3);
    mat4 v = mat4(vc0, vc1, vc2, vc3);
    mat4 p = mat4(pc0, pc1, pc2, pc3);
    v_tex_co = tex_co;
    vec4 world_point = m * vec4(co, 0.0, 1.0);
    world_point = (vec4(position * world_bounds, 0.0, 0.0)) + world_point;

    gl_Position =  p * v * world_point;
}
