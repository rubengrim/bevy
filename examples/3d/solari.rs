//! Demonstrates global-illumination rendering using bevy_solari.

use bevy::{
    prelude::*,
    solari::{
        SolariCamera3dBundle, SolariMaterial, SolariPathTracer, SolariSettings, SolariSupported,
    },
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            (
                solari_not_supported.run_if(not(resource_exists::<SolariSupported>())),
                setup.run_if(resource_exists::<SolariSupported>()),
            ),
        )
        .add_systems(
            Update,
            (swap_modes, add_solari_materials).run_if(resource_exists::<SolariSupported>()),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(SceneBundle {
        scene: asset_server.load("models/cornell_box.glb#Scene0"),
        ..default()
    });

    commands.spawn(SolariCamera3dBundle {
        transform: Transform::from_matrix(Mat4 {
            x_axis: Vec4::new(0.99480534, 0.0, -0.10179563, 0.0),
            y_axis: Vec4::new(-0.019938117, 0.98063105, -0.19484669, 0.0),
            z_axis: Vec4::new(0.09982395, 0.19586414, 0.975537, 0.0),
            w_axis: Vec4::new(0.68394995, 2.2785425, 6.68395, 1.0),
        }),
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

fn swap_modes(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    camera: Query<(Entity, Option<&SolariSettings>), With<Camera>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        let (camera, real_time) = camera.get_single().unwrap();
        if real_time.is_some() {
            commands
                .entity(camera)
                .remove::<SolariSettings>()
                .insert(SolariPathTracer::default());
        } else {
            commands
                .entity(camera)
                .remove::<SolariPathTracer>()
                .insert(SolariSettings::default());
        }
    }
}

fn add_solari_materials(
    mut commands: Commands,
    mut material_events: EventReader<AssetEvent<StandardMaterial>>,
    entites: Query<(Entity, &Handle<StandardMaterial>)>,
    standard_materials: Res<Assets<StandardMaterial>>,
    mut solari_materials: ResMut<Assets<SolariMaterial>>,
) {
    for event in material_events.iter() {
        let handle = match event {
            AssetEvent::Created { handle } => handle,
            _ => continue,
        };

        if let Some(material) = standard_materials.get(handle) {
            let solari_material = solari_materials.add(SolariMaterial {
                base_color: material.base_color,
                base_color_map: material.base_color_texture.clone(),
                emission: if material.emissive == Color::BLACK {
                    None
                } else {
                    Some(material.emissive * 25.0)
                },
            });

            for (entity, entity_mat_h) in entites.iter() {
                if entity_mat_h == handle {
                    commands.entity(entity).insert(solari_material.clone());
                }
            }
        }
    }
}
