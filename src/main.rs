use bevy::gltf::GltfMesh;
use bevy::input::mouse::{AccumulatedMouseMotion, MouseWheel};
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Resource)]
struct GameAssets {
    cube_gltf: Handle<Gltf>,
    cube2_gltf: Handle<Gltf>,
}

const SHADER_ASSET_PATH: &str = "shaders/morph_material.wgsl";

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct MorphExtension {
    #[uniform(100)]
    red_weight: f32,
    #[uniform(100)]
    green_weight: f32,
    #[uniform(100)]
    blue_weight: f32,
}

impl MaterialExtension for MorphExtension {
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                debug_scene_data,
                control_morph_targets,
                control_morph_weights,
                camera_orbit,
            ),
        )
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, MorphExtension>,
        >::default())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, MorphExtension>>>,
) {
    let cube_gltf = asset_server.load("cube_morph_test.glb");
    let cube_scene = asset_server.load("cube_morph_test.glb#Scene0");
    let cube2_gltf = asset_server.load("bruh2.glb");
    let cube2_mesh = asset_server.load("bruh2.glb#Mesh0/Primitive0");

    commands.insert_resource(GameAssets {
        cube_gltf,
        cube2_gltf,
    });

    commands.spawn((SceneRoot(cube_scene), Transform::from_xyz(-1.5, 0.0, 1.5)));

    commands.spawn((
        Mesh3d(cube2_mesh),
        MeshMaterial3d(materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: Color::srgb(0.6, 0.6, 0.0),
                ..default()
            },
            extension: MorphExtension {
                red_weight: 1.0,
                green_weight: 0.0,
                blue_weight: 0.0,
            },
        })),
        Transform::from_xyz(1.5, 0.0, -1.5),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.95, 0.85),
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.7, 0.5, 0.0)),
    ));
}

fn camera_orbit(
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
) {
    let mut camera_transform = camera_query.single_mut().unwrap();

    let delta = mouse_motion.delta;
    let sensitivity = 0.005;

    let radius = camera_transform.translation.length();
    let mut yaw = camera_transform
        .translation
        .z
        .atan2(camera_transform.translation.x);
    let mut pitch = (camera_transform.translation.y / radius).asin();

    yaw -= delta.x * sensitivity;
    pitch -= delta.y * sensitivity;
    pitch = pitch.clamp(-1.5, 1.5);

    camera_transform.translation = Vec3::new(
        radius * pitch.cos() * yaw.cos(),
        radius * pitch.sin(),
        radius * pitch.cos() * yaw.sin(),
    );

    camera_transform.look_at(Vec3::ZERO, Vec3::Y);
}

fn control_morph_targets(
    mut morph_weights_query: Query<&mut MorphWeights>,
    mut scroll_events: EventReader<MouseWheel>,
) {
    let scroll_delta: f32 = scroll_events.read().map(|ev| ev.y).sum();

    if scroll_delta != 0.0 {
        for mut morph_weights in morph_weights_query.iter_mut() {
            if let Some(weight) = morph_weights.weights_mut().first_mut() {
                *weight = (*weight + scroll_delta * 0.05).clamp(0.0, 1.0);
                println!("Morph weight: {:.2}", *weight);
            }
        }
    }
}

fn control_morph_weights(
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, MorphExtension>>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs() * 0.5;

    for (_id, material) in materials.iter_mut() {
        if keyboard.pressed(KeyCode::KeyQ) {
            material.extension.red_weight = (material.extension.red_weight + delta).min(1.0);
        }
        if keyboard.pressed(KeyCode::KeyA) {
            material.extension.red_weight = (material.extension.red_weight - delta).max(0.0);
        }

        if keyboard.pressed(KeyCode::KeyW) {
            material.extension.green_weight = (material.extension.green_weight + delta).min(1.0);
        }
        if keyboard.pressed(KeyCode::KeyS) {
            material.extension.green_weight = (material.extension.green_weight - delta).max(0.0);
        }

        if keyboard.pressed(KeyCode::KeyE) {
            material.extension.blue_weight = (material.extension.blue_weight + delta).min(1.0);
        }
        if keyboard.pressed(KeyCode::KeyD) {
            material.extension.blue_weight = (material.extension.blue_weight - delta).max(0.0);
        }

        if keyboard.just_pressed(KeyCode::Space) {
            println!(
                "Red: {:.2}, Green: {:.2}, Blue: {:.2}",
                material.extension.red_weight,
                material.extension.green_weight,
                material.extension.blue_weight
            );
        }
    }
}

// vibe coded garbage, ignore it
fn debug_scene_data(
    gltf_assets: Res<Assets<Gltf>>,
    scene_assets: Res<Assets<Scene>>,
    gltf_mesh_assets: Res<Assets<GltfMesh>>,
    mesh_assets: Res<Assets<Mesh>>,
    material_assets: Res<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    game_assets: Res<GameAssets>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyQ) {
        if asset_server.is_loaded_with_dependencies(&game_assets.cube_gltf) {
            if let Some(gltf) = gltf_assets.get(&game_assets.cube_gltf) {
                println!("\n=== GLTF Asset Info ===");

                println!("\nScenes: {}", gltf.scenes.len());
                for (i, scene_handle) in gltf.scenes.iter().enumerate() {
                    if let Some(_scene) = scene_assets.get(scene_handle) {
                        println!("  Scene {}", i);
                    }
                }

                println!("\nMeshes: {}", gltf.meshes.len());
                for (i, gltf_mesh_handle) in gltf.meshes.iter().enumerate() {
                    if let Some(gltf_mesh) = gltf_mesh_assets.get(gltf_mesh_handle) {
                        println!(
                            "  GltfMesh {}: {} primitives",
                            i,
                            gltf_mesh.primitives.len()
                        );

                        for (j, primitive) in gltf_mesh.primitives.iter().enumerate() {
                            if let Some(mesh) = mesh_assets.get(&primitive.mesh) {
                                println!("    Primitive {}:", j);
                                println!("      Topology: {:?}", mesh.primitive_topology());
                                println!("      Vertex count: {}", mesh.count_vertices());

                                print!("      Attributes: ");
                                for (attr_id, _) in mesh.attributes() {
                                    print!("{:?} ", attr_id);
                                }
                                println!();

                                if let Some(morph_targets) = mesh.morph_targets() {
                                    println!("      Morph targets handle: {:?}", morph_targets);
                                }

                                if let Some(morph_names) = mesh.morph_target_names() {
                                    println!("      Morph target names: {:?}", morph_names);
                                }

                                if let Some(indices) = mesh.indices() {
                                    println!("      Indices: {} indices", indices.len());
                                } else {
                                    println!("      Indices: None (non-indexed)");
                                }
                            }
                        }
                    }
                }

                println!("\nMaterials: {}", gltf.materials.len());
                for (i, material_handle) in gltf.materials.iter().enumerate() {
                    if let Some(material) = material_assets.get(material_handle) {
                        println!(
                            "  Material {}: base_color={:?}, roughness={}, metallic={}",
                            i,
                            material.base_color,
                            material.perceptual_roughness,
                            material.metallic
                        );
                    }
                }

                println!("\nAnimations: {}", gltf.animations.len());
                if !gltf.animations.is_empty() {
                    println!("  Animation handles: {:?}", gltf.animations);
                }

                println!("\nNodes: {}", gltf.nodes.len());
                println!("===================\n");
            }
        } else {
            println!("Asset still loading...");
        }
    }

    if keyboard.just_pressed(KeyCode::KeyE) {
        if asset_server.is_loaded_with_dependencies(&game_assets.cube2_gltf) {
            if let Some(gltf) = gltf_assets.get(&game_assets.cube2_gltf) {
                println!("\n=== CUBE 2 VERTEX DATA ===");

                for (i, gltf_mesh_handle) in gltf.meshes.iter().enumerate() {
                    if let Some(gltf_mesh) = gltf_mesh_assets.get(gltf_mesh_handle) {
                        for (j, primitive) in gltf_mesh.primitives.iter().enumerate() {
                            if let Some(mesh) = mesh_assets.get(&primitive.mesh) {
                                println!("Mesh {} Primitive {}:", i, j);

                                if let Some(colors) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
                                    use bevy::render::mesh::VertexAttributeValues;
                                    if let VertexAttributeValues::Float32x4(color_data) = colors {
                                        println!("  COLOR_0 ({} vertices):", color_data.len());
                                        for (idx, color) in color_data.iter().take(5).enumerate() {
                                            println!(
                                                "    Vertex {}: R={:.3} G={:.3} B={:.3} A={:.3}",
                                                idx, color[0], color[1], color[2], color[3]
                                            );
                                        }
                                    }
                                }

                                for (attr_id, attr_values) in mesh.attributes() {
                                    println!("  Attribute: {:?}", attr_id);
                                    use bevy::render::mesh::VertexAttributeValues;
                                    if let VertexAttributeValues::Float32x4(color_data) =
                                        attr_values
                                    {
                                        println!("    First 5 vertices:");
                                        for (idx, color) in color_data.iter().take(5).enumerate() {
                                            println!(
                                                "      Vertex {}: R={:.3} G={:.3} B={:.3} A={:.3}",
                                                idx, color[0], color[1], color[2], color[3]
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                println!("===================\n");
            }
        }
    }
}
