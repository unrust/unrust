#ifndef GL_ES
#define varying in
#define gl_FragColor FragColor
#define texture2D texture
out vec4 FragColor;
#endif

varying vec2 vTexCoords;
uniform sampler2D uDepthMap;

void main()
{
    float r = texture2D(uDepthMap, vTexCoords).r;
    vec3 depth = vec3(r);
    vec4 color = vec4(1.0 - depth, 1.0);

    gl_FragColor = color;
}         