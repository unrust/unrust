#ifndef GL_ES
#define attribute in
#define varying out
#define texture2D texture
#endif

attribute vec3 aVertexPosition;
attribute vec3 aVertexNormal;
attribute vec3 aVertexTangent;

attribute vec2 aTextureCoord;

uniform mat4 uMVMatrix;
uniform mat4 uPMatrix;
uniform mat4 uNMatrix;
uniform mat4 uMMatrix;

varying vec3 vFragPos;
varying mat3 vTBN;

varying vec2 vTexCoords;
varying vec3 vNormal;

void main(void) {
    vFragPos = vec3(uMMatrix * vec4(aVertexPosition, 1.0));            
    
    vec3 T = normalize(mat3(uMMatrix) * aVertexTangent);
    vec3 N = normalize(mat3(uMMatrix) * aVertexNormal);    
    vec3 B = cross(T,N);
    
    vNormal = mat3(uNMatrix) * aVertexNormal;   

    vTBN = mat3(T, B, N);
    vTexCoords = aTextureCoord;

    gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 1.0);
}
