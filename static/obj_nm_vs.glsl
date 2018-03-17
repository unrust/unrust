#ifndef GL_ES
#define attribute in
#define varying out
#define texture2D texture
#endif

#define UNI_POINT_LIGHTS 4

attribute vec3 aVertexPosition;
attribute vec3 aVertexNormal;
attribute vec3 aVertexTangent;

attribute vec2 aTextureCoord;

uniform mat4 uMVMatrix;
uniform mat4 uPMatrix;
uniform mat4 uNMatrix;
uniform mat4 uMMatrix;

uniform vec3 uViewPos;

varying vec3 vFragPos;

varying vec2 vTexCoords;
varying vec3 vNormal;

struct DirectionalLightVS {
    vec3 direction;
};

struct PointLightVS {
    vec3 position;
    vec3 direction;
};

uniform DirectionalLightVS uDirectionalLightVS;
uniform PointLightVS uPointLightsVS[UNI_POINT_LIGHTS];

varying DirectionalLightVS vDirectionalLightTgt;
varying PointLightVS vPointLightsTgt[UNI_POINT_LIGHTS];
varying vec3 vViewDirTgt;

void main(void) {
    vec3 vWorldPos = vec3(uMMatrix * vec4(aVertexPosition, 1.0));            
    
    vec3 T = normalize(mat3(uMMatrix) * aVertexTangent);
    vec3 N = normalize(mat3(uMMatrix) * aVertexNormal);    
    vec3 B = cross(T,N);    
    mat3 TBN = transpose(mat3(T, B, N));
    
    vViewDirTgt = TBN * normalize(uViewPos - vWorldPos);
    vNormal = TBN * aVertexNormal;   
    vTexCoords = aTextureCoord;

    vDirectionalLightTgt.direction = TBN * uDirectionalLightVS.direction;

    for(int i = 0; i < UNI_POINT_LIGHTS; i++){
        vPointLightsTgt[i].position = uPointLightsVS[i].position;
        vPointLightsTgt[i].direction = TBN * uPointLightsVS[i].direction;
    }

    vFragPos = vWorldPos;    
    gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 1.0);
}
