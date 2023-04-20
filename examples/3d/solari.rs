//! Demonstrates global-illumination rendering using bevy_solari.

use bevy::{
    prelude::*,
    solari::{SolariCamera3dBundle, SolariMaterial, SolariMaterialMeshBundle, SolariSupported},
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    if app.world.contains_resource::<SolariSupported>() {
        app.add_systems(Startup, setup);
    } else {
        app.add_systems(Startup, solari_not_supported);
    }

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SolariMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(SolariCamera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(SolariMaterialMeshBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(SolariMaterial {
            base_color: Color::rgb(0.3, 0.5, 0.3),
            base_color_map: Some(asset_server.load("branding/bevy_logo_dark_big.png")),
            ..default()
        }),
        ..default()
    });

    commands.spawn(SolariMaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(SolariMaterial {
            base_color: Color::rgb_linear(0.0, 0.0, 0.0),
            emission: Some(Color::rgb_linear(4.0, 4.0, 4.0)),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
}

fn solari_not_supported(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(
        TextBundle::from_section(
            "Current GPU does not support bevy_solari",
            TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 48.0,
                color: Color::WHITE,
            },
        )
        .with_style(Style {
            margin: UiRect::all(Val::Auto),
            ..default()
        }),
    );

    commands.spawn(Camera2dBundle::default());
}
