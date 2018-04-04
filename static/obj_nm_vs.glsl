#define USE_GLSL_300ES

//#ifndef GL_ES
#define attribute in
#define varying out
#define texture2D texture

#define UNI_POINT_LIGHTS 4

#include "unrust/default_uniforms.glsl"

attribute vec3 aVertexPosition;
attribute vec3 aVertexNormal;
attribute vec3 aVertexTangent;
attribute vec2 aTextureCoord;

uniform vec3 uViewPos;

varying vec3 vFragPos;

varying vec2 vTexCoords;
varying vec3 vNormal;
varying vec3 vWorldNormal;

struct DirectionalLightVS {
    vec3 direction;
};

struct PointLightVS {
    vec3 position;
    vec3 direction;
};

uniform DirectionalLightVS uDirectionalLightVS;
uniform PointLightVS uPointLightsVS[UNI_POINT_LIGHTS];

varying vec3 vDirectionalLightDirTgt;
varying vec3 vPointLightPointsTgt[UNI_POINT_LIGHTS];
varying vec3 vViewDirTgt;
varying vec3 vFragPosTgt;

void main(void) {
    vec3 vWorldPos = vec3(uMMatrix * vec4(aVertexPosition, 1.0));            
    vec3 worldNormal = mat3(uMMatrix) * aVertexNormal;

    vec3 T = normalize(mat3(uMMatrix) * aVertexTangent);
    vec3 N = normalize(worldNormal);    
    vec3 B = cross(T,N);    
    mat3 TBN = transpose(mat3(T, B, N));

    vWorldNormal = worldNormal;
    
    vViewDirTgt = TBN * normalize(uViewPos - vWorldPos);
    vNormal = TBN * aVertexNormal;   
    vTexCoords = aTextureCoord;

    vDirectionalLightDirTgt = TBN * uDirectionalLightVS.direction;

    for(int i = 0; i < UNI_POINT_LIGHTS; i++){
        vPointLightPointsTgt[i] = TBN * uPointLightsVS[i].position;
    }
    vFragPosTgt = TBN * vWorldPos;

    vFragPos = vWorldPos;    
    gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 1.0);
}
