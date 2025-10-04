#import bevy_pbr::{
    pbr_fragment::pbr_input_from_vertex_output,
    forward_io::{VertexOutput, FragmentOutput}, 
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_functions::{get_world_from_local, mesh_position_local_to_world, mesh_normal_local_to_world},
    view_transformations::position_world_to_clip,
}

struct MorphMaterial {
    red: f32,
    green: f32,
    blue: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> weights: MorphMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {

    // how much the weights can offset the vertex position
    let offset = 0.5;

    // displacement is packed as r,g,b
    let displacements = (
        (vertex.color.r - offset) * weights.red +
        (vertex.color.g - offset) * weights.green +
        (vertex.color.b - offset) * weights.blue
    );
    let displaced_position = vertex.position + (vertex.normal * displacements);
    
    let model = get_world_from_local(vertex.instance_index);

    var out: VertexOutput;
    out.world_position = mesh_position_local_to_world(model, vec4<f32>(displaced_position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
    out.color = vec4<f32>(1.0, 1.0, 0.0, 1.0); // white since it's just applied multiplicatively in fragment

    // and pass these on i guess
    out.world_normal = mesh_normal_local_to_world(vertex.normal, vertex.instance_index);
    out.uv = vertex.uv;
    out.instance_index = vertex.instance_index;

    return out;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    var pbr_input = pbr_input_from_vertex_output(in, is_front, false);
    
    let x_tangent = dpdx(in.world_position.xyz);
    let y_tangent = dpdy(in.world_position.xyz);
    let flat_normal = normalize(cross(y_tangent, x_tangent));
    pbr_input.N = flat_normal;
    
    var color = apply_pbr_lighting(pbr_input);
    color = main_pass_post_lighting_processing(pbr_input, color);
    return color;
}