
uniform float metallic;
uniform float roughness;
uniform vec3 cameraPosition;

uniform vec4 albedo;
#ifdef USE_ALBEDO_TEXTURE
uniform sampler2D albedoTexture;
uniform mat3 albedoTexTransform;
#endif

uniform vec4 emissive;
#ifdef USE_EMISSIVE_TEXTURE
uniform sampler2D emissiveTexture;
uniform mat3 emissiveTexTransform;
#endif

#ifdef USE_METALLIC_ROUGHNESS_TEXTURE
uniform sampler2D metallicRoughnessTexture;
uniform mat3 metallicRoughnessTexTransform;
#endif

#ifdef USE_OCCLUSION_TEXTURE
uniform sampler2D occlusionTexture;
uniform mat3 occlusionTexTransform;
uniform float occlusionStrength;
#endif

#ifdef USE_NORMAL_TEXTURE
uniform sampler2D normalTexture;
uniform mat3 normalTexTransform;
uniform float normalScale;
#endif

in vec3 pos;
in vec3 nor;
in vec4 col;

layout (location = 0) out vec4 outColor;

#ifdef USE_OIT
layout (location = 1) out float accumAlpha;
#endif

#ifdef USE_OIT
float weight(float z, float a)
{
    // return (1 - z) * 10; // a * (1 + (1 + z));
    // return clamp(pow(abs(z), -5), 1e-2, 3e3);
    // float k = 0.01;
    // return clamp(10 / (1 + 10 * pow(abs(z * k), 5)), 1e-2, 3e3);
    // return 1e-2 * z;
    // return clamp(10 / (1e-5 + pow(abs(z)/5, 2) + pow(abs(z)/200, 6)), 1e-2, 3e3);
    // return clamp(pow(z, -4), 1e-2, 3e3);
    // return clamp(10 / (1e-5 + pow(abs(z)/5, 3) + pow(abs(z)/200, 6)), 1e-2, 3e3);
    // return clamp(0.03 / (1e-5 + pow(abs(z)/200, 4)), 1e-2, 3e3);
    // return clamp(0.03 / (1e-5 + pow(abs(z)/200, 4)), 1e-2, 3e3);
    return clamp(pow(min(1.0, a * 10.0) + 0.01, 3.0) * 1e8 * pow(1.0 - z * 0.9, 3.0), 1e-2, 3e3);
}
#endif  

void main()
{
    vec4 surface_color = albedo * col;
#ifdef USE_ALBEDO_TEXTURE
    vec4 c = texture(albedoTexture, (albedoTexTransform * vec3(uvs, 1.0)).xy);
    #ifdef ALPHACUT
        if (c.a < acut) discard;
    #endif
    surface_color *= c;
#endif

    float metallic_factor = metallic;
    float roughness_factor = roughness;
#ifdef USE_METALLIC_ROUGHNESS_TEXTURE
    vec2 t = texture(metallicRoughnessTexture, (metallicRoughnessTexTransform * vec3(uvs, 1.0)).xy).gb;
    roughness_factor *= t.x;
    metallic_factor *= t.y;
#endif

    float occlusion = 1.0;
#ifdef USE_OCCLUSION_TEXTURE
    occlusion = mix(1.0, texture(occlusionTexture, (occlusionTexTransform * vec3(uvs, 1.0)).xy).r, occlusionStrength);
#endif

    vec3 normal = normalize(gl_FrontFacing ? nor : -nor);
#ifdef USE_NORMAL_TEXTURE
    vec3 tangent = normalize(gl_FrontFacing ? tang : -tang);
    vec3 bitangent = normalize(gl_FrontFacing ? bitang : -bitang);
    mat3 tbn = mat3(tangent, bitangent, normal);
    normal = tbn * ((2.0 * texture(normalTexture, (normalTexTransform * vec3(uvs, 1.0)).xy).xyz - 1.0) * vec3(normalScale, normalScale, 1.0));
#endif

    vec3 total_emissive = emissive.rgb;
#ifdef USE_EMISSIVE_TEXTURE
    total_emissive *= texture(emissiveTexture, (emissiveTexTransform * vec3(uvs, 1.0)).xy).rgb;
#endif

    outColor.rgb = total_emissive + calculate_lighting(cameraPosition, surface_color.rgb, pos, normal, metallic_factor, roughness_factor, occlusion);
    outColor.a = surface_color.a;

#ifdef USE_OIT
    vec4 color = outColor;
    float w = color.a * weight(gl_FragCoord.z, color.a);
    outColor = vec4(color.rgb * w, color.a);
    accumAlpha = w;
#else
    outColor.rgb = tone_mapping(outColor.rgb);
    outColor.rgb = color_mapping(outColor.rgb);
#endif
}