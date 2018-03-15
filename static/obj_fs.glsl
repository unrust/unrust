#ifndef GL_ES
#define varying in
#define gl_FragColor FragColor
#define texture2D texture
out vec4 FragColor;
#endif

#define UNI_POINT_LIGHTS 4

struct DirectionalLight {
    vec3 direction;
  
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct PointLight {
    vec3 position;
    
    float constant;
    float linear;
    float quadratic;
	
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    float rate;
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
};

uniform vec3 uViewPos;
uniform Material uMaterial;

varying vec3 vFragPos;
varying vec2 vTexCoords;       
varying vec3 vNormal;                       

// Lights
uniform DirectionalLight uDirectionalLight;
uniform PointLight uPointLights[UNI_POINT_LIGHTS];

vec3 CalcDirectionalLight(DirectionalLight light, vec3 normal, vec3 viewDir);
vec3 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir);

void main(void) {
    vec3 norm = normalize(vNormal);
    vec3 viewDir = normalize(uViewPos - vFragPos);

    // Directional Light
    vec3 result = CalcDirectionalLight(uDirectionalLight, norm, viewDir);
    
    // Point Lights
    for(int i = 0; i < UNI_POINT_LIGHTS; i++)
        result += CalcPointLight(uPointLights[i], norm, vFragPos, viewDir);

    // float gamma = 2.2;    
    // gl_FragColor = vec4(pow(result, vec3(1.0/gamma)), uMaterial.transparent);           
    gl_FragColor = vec4(result, uMaterial.transparent);           
}

vec3 CalcDirectionalLight(DirectionalLight light, vec3 normal, vec3 viewDir)
{
    // Ambient
    vec3 ambient = light.ambient * uMaterial.ambient * vec3(texture2D(uMaterial.ambient_tex, vTexCoords));

    vec3 lightDir = normalize(-light.direction);  
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 diffuse = light.diffuse * diff * uMaterial.diffuse * vec3(texture2D(uMaterial.diffuse_tex, vTexCoords));

    // specular    
    // Use blinn here
    vec3 halfwayDir = normalize(lightDir + viewDir);  
    float spec = pow(max(dot(normal, halfwayDir), 0.0), uMaterial.shininess);
    
    vec3 specular = light.specular * spec * uMaterial.specular *  vec3(texture2D(uMaterial.specular_tex, vTexCoords));

    return ambient + diffuse + specular;
}

vec3 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir)
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
    vec3 ambient = light.ambient * uMaterial.ambient * vec3(texture2D(uMaterial.ambient_tex, vTexCoords));
    vec3 diffuse = light.diffuse * diff * uMaterial.diffuse * vec3(texture2D(uMaterial.diffuse_tex, vTexCoords));
    vec3 specular = light.specular * spec * uMaterial.specular * vec3(texture2D(uMaterial.specular_tex, vTexCoords));
    
    ambient *= attenuation;
    diffuse *= attenuation;
    specular *= attenuation;
    
    return (ambient + diffuse + specular) * light.rate;        
}
