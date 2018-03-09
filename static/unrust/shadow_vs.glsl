#ifndef GL_ES
#define attribute in
#define varying out
#endif

attribute vec3 aVertexPosition;
attribute vec2 aTextureCoord;
varying vec2 vTexCoords;
uniform mat4 uMMatrix;
            
void main(void) {
    gl_Position = uMMatrix * vec4(aVertexPosition, 1.0);        
    vTexCoords = aTextureCoord;
}