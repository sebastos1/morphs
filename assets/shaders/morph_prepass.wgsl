#import bevy_pbr::{
    mesh_functions::{get_world_from_local, mesh_position_local_to_world, mesh_normal_local_to_world},
    view_transformations::position_world_to_clip,
    prepass_io::FragmentOutput,
}

struct MorphMaterial {
    red: f32,
    green: f32,
    blue: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> weights: MorphMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
}

@vertex
fn prepass_vertex(vertex: Vertex) -> VertexOutput {
    let offset = 0.5;
    let displacements = (
        (vertex.color.r - offset) * weights.red +
        (vertex.color.g - offset) * weights.green +
        (vertex.color.b - offset) * weights.blue
    );
    let displaced_position = vertex.position + (vertex.normal * displacements);
    
    let model = get_world_from_local(vertex.instance_index);
    
    var out: VertexOutput;
    out.position = position_world_to_clip(
        mesh_position_local_to_world(model, vec4<f32>(displaced_position, 1.0)).xyz
    );
    out.world_normal = mesh_normal_local_to_world(vertex.normal, vertex.instance_index);
    
    return out;
}