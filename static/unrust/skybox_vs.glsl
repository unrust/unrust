#ifndef GL_ES
#define attribute in
#define varying out
#endif

uniform mat4 uPVSkyboxMatrix;

attribute vec3 aVertexPosition;
varying vec3 vTexCoords;

void main()
{
    vTexCoords = aVertexPosition;
    gl_Position = (uPVSkyboxMatrix * vec4(aVertexPosition, 1.0)).xyww;
}         