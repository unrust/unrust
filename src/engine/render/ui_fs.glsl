varying vec3 vColor;
varying vec2 vTextureCoord;
uniform sampler2D uDiffuse;

void main(void) {
    gl_FragColor = texture2D(uDiffuse, vec2(vTextureCoord.s, vTextureCoord.t));
}
