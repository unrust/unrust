#ifndef GL_ES
#define varying in
#define gl_FragColor FragColor
#define textureCube texture
out vec4 FragColor;
#endif

varying vec3 vTexCoords;
uniform samplerCube uSkybox;

void main()
{    
    gl_FragColor = textureCube(uSkybox, vTexCoords);
}