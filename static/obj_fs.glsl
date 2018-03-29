#define USE_GLSL_300ES

#define varying in
#define gl_FragColor FragColor
#define texture2D texture
out vec4 FragColor;

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
    color.ambient = uMaterial.ambient * vec3(texture2D(uMaterial.ambient_tex, vTexCoords));
    color.diffuse = uMaterial.diffuse * vec3(texture2D(uMaterial.diffuse_tex, vTexCoords));
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

struct ShadowMap {
    mat4 light_matrix;
    vec2 map_size;
    vec2 range;
    vec2 viewport_offset;
    vec2 viewport_scale;
};

uniform bool uShadowEnabled;
uniform ShadowMap uShadowMap[4];
uniform sampler2D uShadowMapTexture;

float ndc_z() {
    return ((2.0 * gl_FragCoord.z - gl_DepthRange.near - gl_DepthRange.far) /
    (gl_DepthRange.far - gl_DepthRange.near));
}

float ShadowCalculation(vec3 worldPos, vec3 normal, vec3 lightDir)
{
    if (!uShadowEnabled) {
        return 1.0;
    }

    float nz = ndc_z();
    int index = 0;

    if (nz > uShadowMap[3].range.x) {
        index = 3;
    }
    if (nz > uShadowMap[2].range.x) {
        index = 2;
    }
    if (nz > uShadowMap[1].range.x) {
        index = 1;
    }
    
    vec4 posLightSpace = uShadowMap[index].light_matrix * vec4(worldPos, 1.0);
    vec3 projCoords = posLightSpace.xyz / posLightSpace.w;

    // transform ndc to range [0,1]
    projCoords = projCoords * 0.5 + 0.5;
    if (projCoords.z > 1.0)
        return 1.0;
    
    float currentDepth = projCoords.z;

    float bias = max(0.005 * (1.0 - dot(normal, lightDir)), 0.001);
    vec2 texelSize =  1.0 / uShadowMap[index].map_size;

    float shadow = 0.0;
    for(int x = -1; x <= 1; ++x)
    {
        for(int y = -1; y <= 1; ++y)
        {
            vec2 boundProj = projCoords.xy + vec2(x, y)* texelSize;
            vec2 adjBoundProj = uShadowMap[index].viewport_offset + boundProj * uShadowMap[index].viewport_scale;

            float pcfDepth = texture2D(uShadowMapTexture, adjBoundProj).r;
            float partShadow = float(currentDepth - bias > pcfDepth);
            
            partShadow = float(boundProj.x >= 0.0 && boundProj.x <= 1.0 &&
                boundProj.y >= 0.0 && boundProj.y <= 1.0) * partShadow;
            
            shadow += partShadow;
        }       
    }
    
    // vec2 boundProj = projCoords.xy;
    // vec2 adjBoundProj = uShadowMap[index].viewport_offset + boundProj * uShadowMap[index].viewport_scale;

    // float pcfDepth = texture2D(uShadowMapTexture, adjBoundProj).r;
    // float partShadow = currentDepth - bias > pcfDepth ? 1.0 : 0.0;
    
    // if (boundProj.x < 0.0 || boundProj.x > 1.0 ||
    // boundProj.y < 0.0 || boundProj.y > 1.0 ) {
    //     partShadow = 0.0;
    // }            
    
    //shadow = partShadow;
    
    shadow /= 9.0;

    return (1.0 - shadow);
}

vec3 CalcDirectionalLight(DirectionalLight light, vec3 normal, vec3 viewDir, MaterialColor color)
{
    // Ambient
    vec3 ambient = light.ambient * uMaterial.ambient * color.ambient;

    vec3 lightDir = normalize(-light.direction);  
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 diffuse = light.diffuse * diff * uMaterial.diffuse * color.diffuse;

    // specular    
    // Use blinn here
    vec3 halfwayDir = normalize(lightDir + viewDir);  
    float spec = pow(max(dot(normal, halfwayDir), 0.0), uMaterial.shininess);
    
    vec3 specular = light.specular * spec * uMaterial.specular *  color.specular;
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
    vec3 ambient = light.ambient * uMaterial.ambient * color.ambient;
    vec3 diffuse = light.diffuse * diff * uMaterial.diffuse * color.diffuse;
    vec3 specular = light.specular * spec * uMaterial.specular * color.specular;
    
    ambient *= attenuation;
    diffuse *= attenuation;
    specular *= attenuation;
    
    return (ambient + diffuse + specular) * light.rate;        
}
