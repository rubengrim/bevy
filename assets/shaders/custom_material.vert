#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;

layout(location = 0) out vec2 v_Uv;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
    mat4 View;
    mat4 InverseView;
    mat4 Projection;
    vec3 WorldPosition;
    float width;
    float height;
};

struct Mesh {
    mat4 Model;
    mat4 InverseTransposeModel;
    uint flags;
};

#ifdef PER_OBJECT_BUFFER_BATCH_SIZE
layout(set = 2, binding = 0) uniform Mesh Meshes[#{PER_OBJECT_BUFFER_BATCH_SIZE}];
#else
layout(set = 2, binding = 0) readonly buffer _Meshes {
    Mesh Meshes[];
};
#endif // PER_OBJECT_BUFFER_BATCH_SIZE

void main() {
    v_Uv = Vertex_Uv;
    gl_Position = ViewProj
        * Meshes[gl_InstanceIndex].Model
        * vec4(Vertex_Position, 1.0);
}