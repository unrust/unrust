#define USE_GLSL_300ES

#define attribute in
#define varying out

#include "unrust/default_uniforms.glsl"

attribute vec3 aVertexPosition;
attribute vec3 aVertexNormal;
attribute vec2 aTextureCoord;

varying vec3 vFragPos;
varying vec3 vNormal;
varying vec2 vTexCoords;

void main(void) {
    vFragPos = vec3(uMMatrix * vec4(aVertexPosition, 1.0));            
    
    vNormal = mat3(uNMatrix) * aVertexNormal;
    vTexCoords = aTextureCoord;
    
    gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 1.0);
}
