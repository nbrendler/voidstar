in vec2 co;
in vec2 tex_coords;

out vec2 v_tex_coords;

// use these once this bug is fixed:
// https://github.com/phaazon/luminance-rs/issues/434
uniform mat4 model;
uniform mat4 projection;
uniform vec4 mc0;
uniform vec4 mc1;
uniform vec4 mc2;
uniform vec4 mc3;
uniform vec4 pc0;
uniform vec4 pc1;
uniform vec4 pc2;
uniform vec4 pc3;

void main()
{
    mat4 m = mat4(mc0, mc1, mc2, mc3);
    mat4 p = mat4(pc0, pc1, pc2, pc3);
    v_tex_coords = tex_coords;
    gl_Position =  p * m * vec4(co, 0.0, 1.0);
}
