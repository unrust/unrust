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
    sampler2D diffuse;
    float shininess;
};

uniform vec3 uViewPos;
uniform Material uMaterial;

varying vec3 vFragPos;
varying vec2 vTexCoords;       
varying vec3 vNormal;       
varying vec4 vPosLightSpace;

uniform sampler2D uShadowMap;

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

    gl_FragColor = vec4(result, 1.0);           
}

float ShadowCalculation(vec4 posLightSpace)
{
    vec3 projCoords = posLightSpace.xyz / posLightSpace.w;

    // transform ndc to range [0,1
    projCoords = projCoords * 0.5 + 0.5;

    float closestDepth = texture2D(uShadowMap, projCoords.xy).r;

    float currentDepth = projCoords.z;

    float shadow = currentDepth > closestDepth ? 1.0 : 0.0;

    return shadow;
}

vec3 CalcDirectionalLight(DirectionalLight light, vec3 normal, vec3 viewDir)
{
    // diffuse
    vec3 ambient = light.ambient * vec3(texture2D(uMaterial.diffuse, vTexCoords));

    vec3 lightDir = normalize(-light.direction);  
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 diffuse = light.diffuse * diff * texture2D(uMaterial.diffuse, vTexCoords).rgb;  

    // specular    
    vec3 reflectDir = reflect(-lightDir, normal);  
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), uMaterial.shininess);
    vec3 specular = light.specular * spec; 

    float shadow = ShadowCalculation(vPosLightSpace);

    return ambient + (diffuse + specular) * (1.0 - shadow);
}

vec3 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir)
{
    vec3 lightDir = normalize(light.position - fragPos);
    
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), uMaterial.shininess);
    
    // attenuation
    float distance = length(light.position - fragPos);
    float d = (light.constant + light.linear * distance + light.quadratic * (distance * distance));
    float attenuation = 1.0 / max(d, 0.001);
    
    // combine results
    vec3 ambient = light.ambient * vec3(texture2D(uMaterial.diffuse, vTexCoords));
    vec3 diffuse = light.diffuse * diff * vec3(texture2D(uMaterial.diffuse, vTexCoords));
    vec3 specular = light.specular * spec;
    
    ambient *= attenuation;
    diffuse *= attenuation;
    specular *= attenuation;
    
    return (ambient + diffuse + specular) * light.rate;        
}
