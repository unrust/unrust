#ifndef GL_ES
#define varying in
#define gl_FragColor FragColor
#define texture2D texture
out vec4 FragColor;
#endif

#define UNI_POINT_LIGHTS 4

struct DirectionalLight {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct PointLight {
    float constant;
    float linear;
    float quadratic;
	
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    float rate;
};

struct DirectionalLightVS {
    vec3 direction;
};

struct PointLightVS {
    vec3 position;
    vec3 direction;
};

struct Material {
    vec3 ambient;
    sampler2D ambient_tex;
    
    vec3 diffuse;
    sampler2D diffuse_tex;

    vec3 specular;
    sampler2D specular_tex;

    float shininess;
    float transparent;
    sampler2D mask_tex;

    sampler2D normal_map;
};

struct MaterialColor {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

uniform Material uMaterial;

varying vec3 vFragPos;
varying vec2 vTexCoords;       
varying mat3 vTBN;
varying vec3 vNormal;
varying vec3 vViewDirTgt;

varying DirectionalLightVS vDirectionalLightTgt;
varying PointLightVS vPointLightsTgt[UNI_POINT_LIGHTS];

// Lights
uniform DirectionalLight uDirectionalLight;
uniform PointLight uPointLights[UNI_POINT_LIGHTS];


vec3 CalcDirectionalLight(DirectionalLight light, DirectionalLightVS lightTgt, vec3 normal, vec3 viewDir, MaterialColor color);
vec3 CalcPointLight(PointLight light, PointLightVS lightTgt, vec3 normal, vec3 fragPos, vec3 viewDir, MaterialColor color);

uniform bool uNoNormalMap;

/* 
    X: -1 to +1 :  Red: 0 to 255
    Y: -1 to +1 :  Green: 0 to 255
  Z: 0 to -1 :  Blue: 128 to 255
  */

vec3 decode_normalmap(vec3 n) {    
    return vec3( (n.xy * 2.0 - vec2(1.0, 1.0)),  n.z);
}

void main(void) {
    vec3 norm = normalize(vNormal);

    //if(!uNoNormalMap) 
    {
        norm = texture2D(uMaterial.normal_map, vTexCoords ).rgb;
        norm = decode_normalmap(norm);        
    }

    // Presample the color
    MaterialColor color;
    color.ambient = uMaterial.ambient * vec3(texture2D(uMaterial.ambient_tex, vTexCoords));
    color.diffuse = uMaterial.diffuse * vec3(texture2D(uMaterial.diffuse_tex, vTexCoords));
    color.specular = uMaterial.specular * vec3(texture2D(uMaterial.specular_tex, vTexCoords));

    // Directional Light
    vec3 result = CalcDirectionalLight(uDirectionalLight, vDirectionalLightTgt, norm, vViewDirTgt, color);
    
    // Point Lights
    for(int i = 0; i < UNI_POINT_LIGHTS; i++)
        result += CalcPointLight(uPointLights[i], vPointLightsTgt[i], norm, vFragPos, vViewDirTgt, color);

    // float gamma = 2.2;    
    // gl_FragColor = vec4(pow(result, vec3(1.0/gamma)), uMaterial.transparent);           
    gl_FragColor = vec4(result, uMaterial.transparent * texture2D(uMaterial.mask_tex, vTexCoords).r );           

}

vec3 CalcDirectionalLight(DirectionalLight light, DirectionalLightVS lightTgt, vec3 normal, vec3 viewDir, MaterialColor color)
{
    // Ambient
    vec3 ambient = light.ambient * color.ambient;

    vec3 lightDir = normalize(-lightTgt.direction);  
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 diffuse = light.diffuse * diff * color.diffuse;

    // specular    
    // Use blinn here
    vec3 halfwayDir = normalize(lightDir + viewDir);  
    float spec = pow(max(dot(normal, halfwayDir), 0.0), uMaterial.shininess);
    
    vec3 specular = light.specular * spec * color.specular;

    return ambient + diffuse + specular;
}

vec3 CalcPointLight(PointLight light, PointLightVS lightTgt, vec3 normal, vec3 fragPos, vec3 viewDir, MaterialColor color)
{
    vec3 lightDir = lightTgt.direction;
    
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    
    // Use blinn here
    vec3 halfwayDir = normalize(lightDir + viewDir);  
    float spec = pow(max(dot(normal, halfwayDir), 0.0), uMaterial.shininess);
    
    // attenuation
    float distance = length(lightTgt.position - fragPos);
    float d = (light.constant + light.linear * distance + light.quadratic * (distance * distance));
    float attenuation = 1.0 / max(d, 0.001);
    
    // combine results
    vec3 ambient = light.ambient * color.ambient;
    vec3 diffuse = light.diffuse * diff * color.diffuse;
    vec3 specular = light.specular * spec * color.specular;
    
    ambient *= attenuation;
    diffuse *= attenuation;
    specular *= attenuation;
    
    return (ambient + diffuse + specular) * light.rate;        
}
