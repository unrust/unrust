#ifndef GL_ES
#define varying in
#define gl_FragColor FragColor
out vec4 FragColor;
#endif

varying vec3 vColor;
varying vec2 vTextureCoord;
uniform sampler2D uDiffuse;

const float crtBend			= 2.8;
const float crtOverscan		= 0.1;
vec2 crt(vec2 coord)
{
	// put in symmetrical coords
	coord = (coord - 0.5) * 2.0 / (crtOverscan + 1.0);

	coord *= 1.1;

	// deform coords
	coord.x *= 1.0 + pow((abs(coord.y) / crtBend), 2.0);
	coord.y *= 1.0 + pow((abs(coord.x) / crtBend), 2.0);

	// transform back to 0.0 - 1.0 space
	coord  = (coord / 2.0) + 0.5;

	return coord;
}

void main(void) {
    vec2 crtCoords = crt(vTextureCoord.st);
    if (crtCoords.x < 0.0 || crtCoords.x > 1.0 || crtCoords.y < 0.0 || crtCoords.y > 1.0) {
    	gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        float coef=0.8 + abs(sin(600*vTextureCoord.t)) * 0.2;
        gl_FragColor = texture2D(uDiffuse, crtCoords) * vec4(0.8,1.0*coef,0.7,1.0)  ;
    }
}
