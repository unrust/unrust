struct ShadowMap {
    mat4 light_matrix;
    mat4 inv_light_matrix;
    vec2 map_size;
    vec2 range;
    vec2 viewport_offset;
    vec2 viewport_scale;
    float tex_size;
};
