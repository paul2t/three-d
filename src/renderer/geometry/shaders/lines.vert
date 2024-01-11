
uniform mat4 viewProjection;
uniform mat4 modelMatrix;
in vec3 position;

out vec3 pos;

#ifdef USE_VERTEX_COLORS
in vec4 color;
#endif

out vec4 col;

void main()
{
    // *** POSITION ***
    mat4 local2World = modelMatrix;
    
    vec4 worldPosition = local2World * vec4(position, 1.);
    worldPosition /= worldPosition.w;
    gl_Position = viewProjection * worldPosition;

    pos = worldPosition.xyz;

    // *** COLOR ***
    col = vec4(1.0);
#ifdef USE_VERTEX_COLORS 
    col *= color;
#endif
}
