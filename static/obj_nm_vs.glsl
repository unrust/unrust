#define USE_GLSL_300ES

//#ifndef GL_ES
#define attribute in
#define varying out
#define texture2D texture
// #else 
// highp mat3 transpose(in highp mat3 inMatrix) {
//     highp vec3 i0 = inMatrix[0];
//     highp vec3 i1 = inMatrix[1];
//     highp vec3 i2 = inMatrix[2];

//     highp mat3 outMatrix = mat3(
//                  vec3(i0.x, i1.x, i2.x),
//                  vec3(i0.y, i1.y, i2.y),
//                  vec3(i0.z, i1.z, i2.z)
//                  );

//     return outMatrix;
// }
// #endif

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

varying vec3 vDirectionalLightDirTgt;
varying vec3 vPointLightDirsTgt[UNI_POINT_LIGHTS];
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

    vDirectionalLightDirTgt = TBN * uDirectionalLightVS.direction;

    for(int i = 0; i < UNI_POINT_LIGHTS; i++){
        vPointLightDirsTgt[i] = TBN * uPointLightsVS[i].direction;
    }

    vFragPos = vWorldPos;    
    gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 1.0);
}
