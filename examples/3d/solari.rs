//! Demonstrates global-illumination rendering using bevy_solari.

use bevy::{
    prelude::*,
    solari::{
        SolariCamera3dBundle, SolariDebugView, SolariMaterial, SolariPathTracer, SolariSettings,
        SolariSupported,
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
            (update, add_solari_materials).run_if(resource_exists::<SolariSupported>()),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(SceneBundle {
        scene: asset_server.load("models/cornell_box.glb#Scene0"),
        ..default()
    });

    commands.spawn(
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(8.0),
            ..default()
        }),
    );

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

fn solari_not_supported(mut commands: Commands) {
    commands.spawn(
        TextBundle::from_section(
            "Current GPU does not support bevy_solari",
            TextStyle {
                font_size: 48.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            margin: UiRect::all(Val::Auto),
            ..default()
        }),
    );

    commands.spawn(Camera2dBundle::default());
}

fn update(
    mut commands: Commands,
    mut ui: Query<&mut Text>,
    mut camera: Query<(Entity, Option<&mut SolariSettings>), With<Camera>>,
    keys: Res<Input<KeyCode>>,
) {
    let (camera, mut real_time) = camera.single_mut();
    let ui = &mut ui.single_mut().sections[0].value;

    ui.clear();
    if real_time.is_some() {
        ui.push_str("(Space) Realtime\n");
    } else {
        ui.push_str("(Space) Path tracer\n");
    }

    if let Some(solari_settings) = real_time.as_mut() {
        if keys.just_pressed(KeyCode::Key1) {
            solari_settings.debug_view = None;
        }
        if keys.just_pressed(KeyCode::Key2) {
            solari_settings.debug_view = Some(SolariDebugView::Depth);
        }
        if keys.just_pressed(KeyCode::Key3) {
            solari_settings.debug_view = Some(SolariDebugView::WorldNormals);
        }
        if keys.just_pressed(KeyCode::Key4) {
            solari_settings.debug_view = Some(SolariDebugView::MotionVectors);
        }
        if keys.just_pressed(KeyCode::Key5) {
            solari_settings.debug_view = Some(SolariDebugView::BaseColors);
        }
        if keys.just_pressed(KeyCode::Key6) {
            solari_settings.debug_view = Some(SolariDebugView::Irradiance);
        }
        if keys.just_pressed(KeyCode::Key7) {
            solari_settings.debug_view = Some(SolariDebugView::ScreenProbesUnfiltered);
        }
        if keys.just_pressed(KeyCode::Key8) {
            solari_settings.debug_view = Some(SolariDebugView::ScreenProbesFiltered);
        }
        if keys.just_pressed(KeyCode::Key9) {
            solari_settings.debug_view = Some(SolariDebugView::WorldCacheIrradiance);
        }

        ui.push_str("\nDebug view:\n");
        if solari_settings.debug_view == None {
            ui.push_str("*1* None\n");
        } else {
            ui.push_str(" 1  None\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::Depth) {
            ui.push_str("*2* Depth\n");
        } else {
            ui.push_str(" 2  Depth\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::WorldNormals) {
            ui.push_str("*3* WorldNormals\n");
        } else {
            ui.push_str(" 3  WorldNormals\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::MotionVectors) {
            ui.push_str("*4* MotionVectors\n");
        } else {
            ui.push_str(" 4  MotionVectors\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::BaseColors) {
            ui.push_str("*5* BaseColors\n");
        } else {
            ui.push_str(" 5  BaseColors\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::Irradiance) {
            ui.push_str("*6* Irradiance\n");
        } else {
            ui.push_str(" 6  Irradiance\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::ScreenProbesUnfiltered) {
            ui.push_str("*7* ScreenProbesUnfiltered\n");
        } else {
            ui.push_str(" 7  ScreenProbesUnfiltered\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::ScreenProbesFiltered) {
            ui.push_str("*8* ScreenProbesFiltered\n");
        } else {
            ui.push_str(" 8  ScreenProbesFiltered\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::WorldCacheIrradiance) {
            ui.push_str("*9* WorldCacheIrradiance\n");
        } else {
            ui.push_str(" 9  WorldCacheIrradiance\n");
        }
    }

    if keys.just_pressed(KeyCode::Space) {
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
