use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::input::mouse::{AccumulatedMouseMotion, MouseWheel};
use bevy::mesh::{Mesh, MeshVertexBufferLayoutRef};
use bevy::pbr::{ExtendedMaterial, MaterialExtension, OpaqueRendererMethod};
use bevy::pbr::{MaterialExtensionKey, MaterialExtensionPipeline};
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

const SHADER_ASSET_PATH: &str = "shaders/morph.wgsl";

#[derive(ShaderType, Clone, Copy, Debug, Reflect, Component)]
struct MyMorphWeights {
    red: f32,
    green: f32,
    blue: f32,
}

#[derive(Asset, AsBindGroup, TypePath, Clone)]
struct MorphExtension {
    #[uniform(100)]
    weights: MyMorphWeights,
}

impl MaterialExtension for MorphExtension {
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        "shaders/morph_prepass.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

type MorphMaterial = ExtendedMaterial<StandardMaterial, MorphExtension>;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FpsOverlayPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                setup_player_materials,
                control_morph_targets,
                control_morph_weights,
                camera_orbit,
                rotate_light,
            ),
        )
        .add_plugins(MaterialPlugin::<MorphMaterial>::default())
        .run();
}

#[derive(Component)]
struct RotatingLight {
    radius: f32,
    speed: f32,
    angle: f32,
}

#[derive(Component)]
struct PlayerCharacter;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<MorphMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube1 = asset_server.load("cube_morph_test.glb#Scene0");
    let pole = asset_server.load("pole.glb#Scene0");
    let cube2 = asset_server.load("bruh2.glb#Mesh0/Primitive0");
    let playa = asset_server.load("playa.glb#Scene0");

    commands.spawn((
        SceneRoot(playa),
        Transform::from_xyz(1.5, 0.0, 1.5).with_scale(Vec3::ONE * 2.0),
        PlayerCharacter,
    ));

    commands.spawn((SceneRoot(cube1), Transform::from_xyz(-1.5, 0.0, 1.5)));

    commands.spawn((SceneRoot(pole), Transform::from_xyz(-1.5, 0.0, -1.5)));

    commands.spawn((
        Mesh3d(cube2),
        MeshMaterial3d(materials.add(MorphMaterial {
            base: StandardMaterial {
                base_color_texture: Some(asset_server.load("image.png")),
                opaque_render_method: OpaqueRendererMethod::Forward,
                ..default()
            },
            extension: MorphExtension {
                weights: MyMorphWeights {
                    red: 1.0,
                    green: 0.0,
                    blue: 0.0,
                },
            },
        })),
        Transform::from_xyz(1.5, 0.0, -1.5)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Plane3d::default().mesh().size(10.0, 10.0)))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.6, 0.6),
            ..default()
        })),
        Transform::from_xyz(0.0, -1.0, 0.0),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.95, 0.85),
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.7, 0.5, 0.0)),
        RotatingLight {
            radius: 7.0,
            speed: 0.5,
            angle: 0.0,
        },
    ));
}

fn setup_player_materials(
    mut commands: Commands,
    player_query: Query<Entity, With<PlayerCharacter>>,
    children_query: Query<&Children>,
    mesh_query: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>),
        Without<MeshMaterial3d<MorphMaterial>>,
    >,
    mut morph_materials: ResMut<Assets<MorphMaterial>>,
    standard_materials: Res<Assets<StandardMaterial>>,
) {
    for player_entity in &player_query {
        for descendant in children_query.iter_descendants(player_entity) {
            if let Ok((entity, material_handle)) = mesh_query.get(descendant) {
                // Preserve the original material's properties
                let base_material = if let Some(mat) = standard_materials.get(&material_handle.0) {
                    mat.clone()
                } else {
                    StandardMaterial::default()
                };

                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(morph_materials.add(MorphMaterial {
                        base: base_material,
                        extension: MorphExtension {
                            weights: MyMorphWeights {
                                red: 0.0,
                                green: 0.0,
                                blue: 0.0,
                            },
                        },
                    })));
            }
        }
    }
}

fn rotate_light(
    mut light_query: Query<(&mut Transform, &mut RotatingLight), With<DirectionalLight>>,
    time: Res<Time>,
) {
    for (mut transform, mut rotating) in light_query.iter_mut() {
        rotating.angle += rotating.speed * time.delta_secs();

        let x = rotating.radius * rotating.angle.cos();
        let z = rotating.radius * rotating.angle.sin();
        let y = 5.0; // Keep height constant

        transform.translation = Vec3::new(x, y, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
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
    mut scroll_events: MessageReader<MouseWheel>,
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
    mut materials: ResMut<Assets<MorphMaterial>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs() * 0.9;

    for (_id, material) in materials.iter_mut() {
        if keyboard.pressed(KeyCode::KeyQ) {
            material.extension.weights.red = (material.extension.weights.red + delta).min(1.0);
        }
        if keyboard.pressed(KeyCode::KeyA) {
            material.extension.weights.red = (material.extension.weights.red - delta).max(0.0);
        }

        if keyboard.pressed(KeyCode::KeyW) {
            material.extension.weights.green = (material.extension.weights.green + delta).min(1.0);
        }
        if keyboard.pressed(KeyCode::KeyS) {
            material.extension.weights.green = (material.extension.weights.green - delta).max(0.0);
        }

        if keyboard.pressed(KeyCode::KeyE) {
            material.extension.weights.blue = (material.extension.weights.blue + delta).min(1.0);
        }
        if keyboard.pressed(KeyCode::KeyD) {
            material.extension.weights.blue = (material.extension.weights.blue - delta).max(0.0);
        }

        if keyboard.just_pressed(KeyCode::Space) {
            println!(
                "Red: {:.2}, Green: {:.2}, Blue: {:.2}",
                material.extension.weights.red,
                material.extension.weights.green,
                material.extension.weights.blue
            );
        }
    }
}
