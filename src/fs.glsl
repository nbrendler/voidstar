out vec4 color;

uniform vec3 v_color;

void main()
{
    color = vec4(v_color, 1.0);
}
