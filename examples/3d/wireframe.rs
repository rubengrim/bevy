//! Showcases wireframe rendering.

use bevy::{
    pbr::wireframe::{Wireframe, WireframeColor, WireframeConfig, WireframePlugin},
    prelude::*,
    render::{render_resource::WgpuFeatures, settings::WgpuSettings},
};

fn main() {
    App::new()
        // When rendering wireframes, you need to enable the POLYGON_MODE_LINE feature of wgpu
        .insert_resource(WgpuSettings {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        // This plugin is required if you want to render wireframes
        .add_plugin(WireframePlugin)
        // Wireframes can be configured with this resource
        // See the associated docs for more details
        .insert_resource(WireframeConfig {
            global: true,
            color: Color::GREEN,
        })
        .add_startup_system(setup)
        .add_system(update_colors)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    // cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        // This enables wireframe drawing on this entity
        .insert(Wireframe)
        // This lets you configure the wireframe color of this entity.
        // If not set, this will use the color in `WireframeConfig`
        .insert(WireframeColor { color: Color::PINK });

    // cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(1.0, 0.5, 0.0),
            ..default()
        })
        // This enables wireframe drawing on this entity.
        // Since no color is specified it will use the global color.
        .insert(Wireframe);

    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

/// This system let's you toggle various wireframe settings
fn update_colors(
    keyboard_input: Res<Input<KeyCode>>,
    mut config: ResMut<WireframeConfig>,
    mut wireframe_colors: Query<&mut WireframeColor>,
) {
    // Toggle showing a wireframe on all meshes
    if keyboard_input.just_pressed(KeyCode::Z) {
        info!("toggle global");
        config.global = !config.global;
    }

    // Toggle the global wireframe color
    if keyboard_input.just_pressed(KeyCode::X) {
        info!("toggle global color");
        config.color = if config.color == Color::GREEN {
            Color::WHITE
        } else {
            Color::GREEN
        };
    }

    // Toggle the color of a wireframe using WireframeColor and not the global color
    if keyboard_input.just_pressed(KeyCode::C) {
        info!("toggle color");
        for mut color in &mut wireframe_colors {
            color.color = if color.color == Color::PINK {
                Color::YELLOW
            } else {
                Color::PINK
            };
        }
    }
}
