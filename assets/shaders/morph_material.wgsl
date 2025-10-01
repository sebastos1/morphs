#import bevy_pbr::{
    mesh_functions,
    forward_io::{Vertex, VertexOutput},
    view_transformations::position_world_to_clip,
    pbr_fragment::pbr_input_from_standard_material,
    forward_io::FragmentOutput,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}

struct MorphMaterial {
    red_weight: f32,
    green_weight: f32,
    blue_weight: f32,
}

@group(2) @binding(100)
var<uniform> morph_material: MorphMaterial;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    // displacement is packed as r,g,b
    let red_offset = vertex.color.r - 0.5;
    let green_offset = vertex.color.g - 0.5;
    let blue_offset = vertex.color.b - 0.5;

    let displacement = (red_offset * morph_material.red_weight) + 
                      (green_offset * morph_material.green_weight) + 
                      (blue_offset * morph_material.blue_weight);
    let displaced_position = vertex.position + (vertex.normal * displacement);
    
    var model = mesh_functions::get_world_from_local(vertex.instance_index);
    out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(displaced_position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
    out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal, vertex.instance_index);
    out.uv = vertex.uv;
    out.instance_index = vertex.instance_index;
    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    return out;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    
    // flat shading didnt work, so we're compensating by calculating the flat normal >:(
    let x_tangent = dpdx(in.world_position.xyz);
    let y_tangent = dpdy(in.world_position.xyz);
    let flat_normal = normalize(cross(y_tangent, x_tangent));
    
    // override the smooth normal with the flat one
    pbr_input.N = flat_normal;
    pbr_input.world_normal = flat_normal;
    
    // where the magic happens
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}