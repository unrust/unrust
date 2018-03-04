#ifndef GL_ES
#define varying in
#define gl_FragColor FragColor
out vec4 FragColor;
#define texture2D texture
#endif

varying vec3 vColor;
varying vec2 vTextureCoord;
uniform sampler2D uDiffuse;

void main(void) {
    gl_FragColor = texture2D(uDiffuse, vec2(vTextureCoord.s, vTextureCoord.t));
}
