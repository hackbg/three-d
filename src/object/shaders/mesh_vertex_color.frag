
in vec4 col;

layout (location = 0) out vec4 outColor;

void main()
{
    outColor = vec4(srgb_from_rgb(col.rgb/255.0), col.a/255.0);
}