//! A shader and a material that uses it.

use bevy::{
    core_pipeline::core_3d::PrepassSettings,
    pbr::PrepassPlugin,
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use fly_cam::{fly_camera, FlyCam};

fn main() {
    App::new()
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<CustomMaterial>::default())
        .add_plugin(MaterialPlugin::<DepthMaterial>::default())
        .add_plugin(PrepassPlugin::<CustomMaterial>::default())
        .add_plugin(PrepassPlugin::<StandardMaterial>::default())
        .add_startup_system(setup)
        .add_system(rotate)
        .add_system(fly_camera)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
    mut depth_materials: ResMut<Assets<DepthMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: std_materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    // depth plane
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Quad {
            flip: false,
            size: Vec2::new(2.0, 2.0),
        })),
        material: depth_materials.add(DepthMaterial {}),
        transform: Transform::from_xyz(-1.0, 1.0, 2.0)
            .looking_at(Vec3::new(2.0, -2.5, -5.0), Vec3::Y),
        ..default()
    });

    //cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: std_materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(-1.0, 0.5, 0.0),
            ..default()
        },
        Rotates,
    ));

    //cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: std_materials.add(StandardMaterial {
            alpha_mode: AlphaMode::Mask(1.0),
            base_color_texture: Some(asset_server.load("branding/icon.png")),
            ..default()
        }),

        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });

    // cube
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(CustomMaterial {
            color: Vec3::ONE,
            color_texture: Some(asset_server.load("branding/icon.png")),
            alpha_mode: AlphaMode::Opaque,
        }),
        transform: Transform::from_xyz(1.0, 0.5, 0.0),
        ..default()
    });

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-3.0, 3., 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PrepassSettings::default(),
        FlyCam,
    ));
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct CustomMaterial {
    #[uniform(0)]
    color: Vec3,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,
    alpha_mode: AlphaMode,
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn prepass_fragment_shader() -> ShaderRef {
        "shaders/red.wgsl".into()
    }
}

#[derive(Component)]
struct Rotates;

fn rotate(mut q: Query<&mut Transform, With<Rotates>>, time: Res<Time>) {
    for mut t in q.iter_mut() {
        let rot =
            (time.seconds_since_startup().sin() * 0.5 + 0.5) as f32 * std::f32::consts::PI * 2.0;
        t.rotation = Quat::from_rotation_z(rot);
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "0af99895-b96e-4451-bc12-c6b1c1c52750"]
pub struct DepthMaterial {}

impl Material for DepthMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/show_depth.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

mod fly_cam {
    use bevy::{input::mouse::MouseMotion, prelude::*};

    #[derive(Component, Default)]
    pub struct FlyCam;

    pub fn fly_camera(
        time: Res<Time>,
        mut camera_transform: Query<&mut Transform, With<FlyCam>>,
        windows: Res<Windows>,
        mouse_input: Res<Input<MouseButton>>,
        key_input: Res<Input<KeyCode>>,
        mut mouse_motion: EventReader<MouseMotion>,
        mut velocity: Local<Vec3>,
    ) {
        if !mouse_input.pressed(MouseButton::Right) {
            return;
        }

        let dt = time.delta_seconds();

        let mut transform = camera_transform.single_mut();

        // Rotate

        let mut mouse_delta = Vec2::ZERO;
        for mouse_motion in mouse_motion.iter() {
            mouse_delta += mouse_motion.delta;
        }

        if mouse_delta != Vec2::ZERO {
            let window = if let Some(window) = windows.get_primary() {
                Vec2::new(window.width() as f32, window.height() as f32)
            } else {
                Vec2::ZERO
            };
            let delta_x = mouse_delta.x / window.x * std::f32::consts::PI * 2.0;
            let delta_y = mouse_delta.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation *= pitch; // rotate around local x axis
        }

        // Translate

        let mut axis_input = Vec3::ZERO;
        if key_input.pressed(KeyCode::W) {
            axis_input.z += 1.0;
        }
        if key_input.pressed(KeyCode::S) {
            axis_input.z -= 1.0;
        }
        if key_input.pressed(KeyCode::D) {
            axis_input.x += 1.0;
        }
        if key_input.pressed(KeyCode::A) {
            axis_input.x -= 1.0;
        }
        if key_input.pressed(KeyCode::Space) {
            axis_input.y += 1.0;
        }
        if key_input.pressed(KeyCode::LShift) {
            axis_input.y -= 1.0;
        }

        if axis_input != Vec3::ZERO {
            let max_speed = 5.0;
            *velocity = axis_input.normalize() * max_speed;
        } else {
            let friction = 0.5;
            *velocity *= 1.0 - friction;
            if velocity.length_squared() < 1e-6 {
                *velocity = Vec3::ZERO;
            }
        }

        let forward = transform.forward();
        let right = transform.right();
        transform.translation +=
            velocity.x * dt * right + velocity.y * dt * Vec3::Y + velocity.z * dt * forward;
    }
}
