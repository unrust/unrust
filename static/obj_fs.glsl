#define USE_GLSL_300ES

#define varying in
#define gl_FragColor FragColor
#define texture2D texture
out vec4 FragColor;

#define UNI_POINT_LIGHTS 4

#include "unrust/phong_light.glsl"
#include "unrust/shadow_utils.glsl"

struct Material {
    vec3 ambient;    
    vec3 diffuse;
    sampler2D diffuse_tex;

    vec3 specular;
    sampler2D specular_tex;

    float shininess;
    float transparent;
    sampler2D mask_tex;
};


struct MaterialColor {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};


uniform vec3 uViewPos;
uniform Material uMaterial;

varying vec3 vFragPos;
varying vec2 vTexCoords;       
varying vec3 vNormal; 
                      

// Lights
uniform DirectionalLight uDirectionalLight;
uniform PointLight uPointLights[UNI_POINT_LIGHTS];

vec3 CalcDirectionalLight(DirectionalLight light, vec3 normal, vec3 viewDir, MaterialColor color);
vec3 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir, MaterialColor color);

void main(void) {
    vec3 norm = normalize(vNormal);
    vec3 viewDir = normalize(uViewPos - vFragPos);

    // Presample the color
    MaterialColor color;
    vec3 diffuse = vec3(texture2D(uMaterial.diffuse_tex, vTexCoords));

    color.ambient = uMaterial.ambient * diffuse;
    color.diffuse = uMaterial.diffuse * diffuse;
    color.specular = uMaterial.specular * vec3(texture2D(uMaterial.specular_tex, vTexCoords));

    // Directional Light
    vec3 result = CalcDirectionalLight(uDirectionalLight, norm, viewDir, color);
    
    // Point Lights
    for(int i = 0; i < UNI_POINT_LIGHTS; i++)
        result += CalcPointLight(uPointLights[i], norm, vFragPos, viewDir, color);

    // float gamma = 2.2;    
    // gl_FragColor = vec4(pow(result, vec3(1.0/gamma)), uMaterial.transparent);           
    gl_FragColor = vec4(result, uMaterial.transparent * texture2D(uMaterial.mask_tex, vTexCoords).r );           
    //gl_FragColor = vec4(0.0, 0.0, 1.0, 1.0);
}

vec3 CalcDirectionalLight(DirectionalLight light, vec3 normal, vec3 viewDir, MaterialColor color)
{
    // Ambient
    vec3 ambient = light.ambient * color.ambient;

    vec3 lightDir = normalize(-light.direction);  
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 diffuse = light.diffuse * diff * color.diffuse;

    // specular    
    // Use blinn here
    vec3 halfwayDir = normalize(lightDir + viewDir);  
    float spec = pow(max(dot(normal, halfwayDir), 0.0), uMaterial.shininess);
    
    vec3 specular = light.specular * spec *  color.specular;
    float shadow = ShadowCalculation(vFragPos, normal, lightDir);

    return ambient + (diffuse + specular) * shadow;
}

vec3 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir, MaterialColor color)
{
    vec3 lightDir = normalize(light.position - fragPos);
    
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    
    // Use blinn here
    vec3 halfwayDir = normalize(lightDir + viewDir);  
    float spec = pow(max(dot(normal, halfwayDir), 0.0), uMaterial.shininess);
    
    // attenuation
    float distance = length(light.position - fragPos);
    float d = (light.constant + light.linear * distance + light.quadratic * (distance * distance));
    float attenuation = 1.0 / max(d, 0.001);
    
    // combine results
    vec3 ambient = light.ambient * color.ambient;
    vec3 diffuse = light.diffuse * diff * color.diffuse;
    vec3 specular = light.specular * spec * color.specular;
    
    return (ambient + diffuse + specular) * attenuation * light.rate;        
}
