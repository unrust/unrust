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

    int i3 = 3 * int(nz > uShadowMap[3].range.x);
    int i2 = max(i3, 2 * int(nz > uShadowMap[2].range.x));
    int index = max(i2, 1 * int(nz > uShadowMap[1].range.x));
    
    vec4 posLightSpace = uShadowMap[index].light_matrix * vec4(worldPos, 1.0);
    vec3 projCoords = posLightSpace.xyz / posLightSpace.w;

    // transform ndc to range [0,1]
    projCoords = projCoords * 0.5 + 0.5;    
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

