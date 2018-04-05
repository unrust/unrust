#include "unrust/shadow_map.glsl"

uniform bool uShadowEnabled;
uniform ShadowMap uShadowMap[4];
uniform sampler2D uShadowMapTexture;

float ndc_z() {
    return ((2.0 * gl_FragCoord.z - gl_DepthRange.near - gl_DepthRange.far) /
    (gl_DepthRange.far - gl_DepthRange.near));
}

vec2 get_shadow_offsets(vec3 N, vec3 L) {
    float cos_alpha = clamp(dot(N, L), 0.0, 1.0);
    float offset_scale_N = sqrt(1.0 - cos_alpha*cos_alpha); // sin(acos(L·N))
    float offset_scale_L = offset_scale_N / cos_alpha;    // tan(acos(L·N))
    return vec2(offset_scale_N, min(2.0, offset_scale_L));
}

vec3 LightSpacePosition(int index, vec3 worldPos, vec3 worldNormal, vec2 bias_offset)
{
    float normal_bias = 0.02;

    vec4 posLightSpace = uShadowMap[index].light_matrix * vec4(worldPos + worldNormal * normal_bias * bias_offset.x, 1.0);
    vec3 projCoordsNDC = posLightSpace.xyz / posLightSpace.w;
    
    // transform ndc to range [0,1]
    return projCoordsNDC * 0.5 + 0.5;
}

int ShadowIndex() {
    float nz = ndc_z();
    int i3 = 3 * int(nz > uShadowMap[3].range.x);
    int i2 = max(i3, 2 * int(nz > uShadowMap[2].range.x));
    
    return max(i2, 1 * int(nz > uShadowMap[1].range.x));
}

float ShadowCalculation(vec3 worldPos, vec3 worldNormal, vec3 normal, vec3 lightDir)
{
    if (!uShadowEnabled) {
        return 1.0;
    }
    
    float constant_bias = 0.5;
    float slope_bias = 3.0;
    float normal_bias = 0.02;

    vec2 bias_offset = get_shadow_offsets(normal, lightDir);    
    int index = ShadowIndex();
            
    vec3 projCoords = LightSpacePosition(index, worldPos, worldNormal, bias_offset);
    float currentDepth = projCoords.z;    
    float texelSize = uShadowMap[index].tex_size;

    float shadow = 0.0;
    float bias = constant_bias * texelSize * (constant_bias + slope_bias * bias_offset.y);

    for(int x = -1; x <= 1; ++x)
    {
        for(int y = -1; y <= 1; ++y)
        {
            vec2 offset = vec2(x, y)* texelSize;
            vec2 boundProj = projCoords.xy + offset;
            vec2 adjBoundProj = uShadowMap[index].viewport_offset + boundProj * uShadowMap[index].viewport_scale;

            float pcfDepth = texture2D(uShadowMapTexture, adjBoundProj).r;
            float partShadow = float(currentDepth - bias > pcfDepth);
            
            partShadow = float(boundProj.x >= 0.0 && boundProj.x <= 1.0 &&
                boundProj.y >= 0.0 && boundProj.y <= 1.0) * partShadow;
            
            shadow += partShadow;
        }       
    }
    
    shadow /= 9.0;

    return (1.0 - shadow);
}

