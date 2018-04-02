#ifndef GL_ES
#define attribute in
#define varying out
#endif

#include "unrust/default_uniforms.glsl"

attribute vec3 aVertexPosition;
attribute vec2 aTextureCoord;
varying vec2 vTextureCoord;
            
void main(void) {
    gl_Position = uMMatrix * vec4(aVertexPosition, 1.0);        
    vTextureCoord = aTextureCoord;
}