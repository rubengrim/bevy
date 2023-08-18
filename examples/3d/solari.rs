//! Demonstrates global-illumination rendering using bevy_solari.

use bevy::{
    prelude::*,
    solari::{
        SolariCamera3dBundle, SolariDebugView, SolariMaterial, SolariPathTracer, SolariSettings,
        SolariSun, SolariSupported,
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
            (update, add_solari_materials, camera_controller)
                .run_if(resource_exists::<SolariSupported>()),
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

    commands.spawn(SolariSun::default());

    commands.spawn((
        SolariCamera3dBundle {
            transform: Transform::from_matrix(Mat4 {
                x_axis: Vec4::new(0.99480534, 0.0, -0.10179563, 0.0),
                y_axis: Vec4::new(-0.019938117, 0.98063105, -0.19484669, 0.0),
                z_axis: Vec4::new(0.09982395, 0.19586414, 0.975537, 0.0),
                w_axis: Vec4::new(0.68394995, 2.2785425, 6.68395, 1.0),
            }),
            ..default()
        },
        CameraController::default(),
    ));
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
        if keys.just_pressed(KeyCode::Key0) {
            solari_settings.debug_view = None;
        }
        if keys.just_pressed(KeyCode::Key1) {
            solari_settings.debug_view = Some(SolariDebugView::Depth);
        }
        if keys.just_pressed(KeyCode::Key2) {
            solari_settings.debug_view = Some(SolariDebugView::WorldNormals);
        }
        if keys.just_pressed(KeyCode::Key3) {
            solari_settings.debug_view = Some(SolariDebugView::MotionVectors);
        }
        if keys.just_pressed(KeyCode::Key4) {
            solari_settings.debug_view = Some(SolariDebugView::BaseColors);
        }
        if keys.just_pressed(KeyCode::Key5) {
            solari_settings.debug_view = Some(SolariDebugView::WorldCacheIrradiance);
        }
        if keys.just_pressed(KeyCode::Key6) {
            solari_settings.debug_view = Some(SolariDebugView::ScreenProbesUnfiltered);
        }
        if keys.just_pressed(KeyCode::Key7) {
            solari_settings.debug_view = Some(SolariDebugView::ScreenProbesFiltered);
        }
        if keys.just_pressed(KeyCode::Key8) {
            solari_settings.debug_view = Some(SolariDebugView::DirectLight);
        }
        if keys.just_pressed(KeyCode::Key9) {
            solari_settings.debug_view = Some(SolariDebugView::IndirectLight);
        }

        ui.push_str("\nDebug view:\n");
        if solari_settings.debug_view == None {
            ui.push_str("*0* None\n");
        } else {
            ui.push_str(" 0  None\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::Depth) {
            ui.push_str("*1* Depth\n");
        } else {
            ui.push_str(" 1  Depth\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::WorldNormals) {
            ui.push_str("*2* WorldNormals\n");
        } else {
            ui.push_str(" 2  WorldNormals\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::MotionVectors) {
            ui.push_str("*3* MotionVectors\n");
        } else {
            ui.push_str(" 3  MotionVectors\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::BaseColors) {
            ui.push_str("*4* BaseColors\n");
        } else {
            ui.push_str(" 4  BaseColors\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::WorldCacheIrradiance) {
            ui.push_str("*5* WorldCacheIrradiance\n");
        } else {
            ui.push_str(" 5  WorldCacheIrradiance\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::ScreenProbesUnfiltered) {
            ui.push_str("*6* ScreenProbesUnfiltered\n");
        } else {
            ui.push_str(" 6  ScreenProbesUnfiltered\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::ScreenProbesFiltered) {
            ui.push_str("*7* ScreenProbesFiltered\n");
        } else {
            ui.push_str(" 7  ScreenProbesFiltered\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::DirectLight) {
            ui.push_str("*8* DirectLight\n");
        } else {
            ui.push_str(" 8  DirectLight\n");
        }
        if solari_settings.debug_view == Some(SolariDebugView::IndirectLight) {
            ui.push_str("*9* IndirectLight\n");
        } else {
            ui.push_str(" 9  IndirectLight\n");
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
                normal_map: material.normal_map_texture.clone(),
                emission: if material.emissive == Color::rgba_linear(0.0, 0.0, 0.0, 1.0) {
                    None
                } else {
                    Some(material.emissive * 25.0)
                },
                emission_map: material.emissive_texture.clone(),
            });

            for (entity, entity_mat_h) in entites.iter() {
                if entity_mat_h == handle {
                    commands.entity(entity).insert(solari_material.clone());
                }
            }
        }
    }
}

// --------------------------------------------------------------------------------------

use bevy::input::mouse::MouseMotion;
use bevy::window::CursorGrabMode;
use std::f32::consts::*;

pub const RADIANS_PER_DOT: f32 = 1.0 / 180.0;

#[derive(Component)]
pub struct CameraController {
    pub enabled: bool,
    pub initialized: bool,
    pub sensitivity: f32,
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_run: KeyCode,
    pub mouse_key_enable_mouse: MouseButton,
    pub keyboard_key_enable_mouse: KeyCode,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocity: Vec3,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            initialized: false,
            sensitivity: 1.0,
            key_forward: KeyCode::W,
            key_back: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::E,
            key_down: KeyCode::Q,
            key_run: KeyCode::LShift,
            mouse_key_enable_mouse: MouseButton::Left,
            keyboard_key_enable_mouse: KeyCode::M,
            walk_speed: 5.0,
            run_speed: 15.0,
            friction: 0.5,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
        }
    }
}

fn camera_controller(
    time: Res<Time>,
    mut windows: Query<&mut Window>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_button_input: Res<Input<MouseButton>>,
    key_input: Res<Input<KeyCode>>,
    mut move_toggled: Local<bool>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
) {
    let dt = time.delta_seconds();

    if let Ok((mut transform, mut options)) = query.get_single_mut() {
        if !options.initialized {
            let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
            options.yaw = yaw;
            options.pitch = pitch;
            options.initialized = true;
        }
        if !options.enabled {
            return;
        }

        // Handle key input
        let mut axis_input = Vec3::ZERO;
        if key_input.pressed(options.key_forward) {
            axis_input.z += 1.0;
        }
        if key_input.pressed(options.key_back) {
            axis_input.z -= 1.0;
        }
        if key_input.pressed(options.key_right) {
            axis_input.x += 1.0;
        }
        if key_input.pressed(options.key_left) {
            axis_input.x -= 1.0;
        }
        if key_input.pressed(options.key_up) {
            axis_input.y += 1.0;
        }
        if key_input.pressed(options.key_down) {
            axis_input.y -= 1.0;
        }
        if key_input.just_pressed(options.keyboard_key_enable_mouse) {
            *move_toggled = !*move_toggled;
        }

        // Apply movement update
        if axis_input != Vec3::ZERO {
            let max_speed = if key_input.pressed(options.key_run) {
                options.run_speed
            } else {
                options.walk_speed
            };
            options.velocity = axis_input.normalize() * max_speed;
        } else {
            let friction = options.friction.clamp(0.0, 1.0);
            options.velocity *= 1.0 - friction;
            if options.velocity.length_squared() < 1e-6 {
                options.velocity = Vec3::ZERO;
            }
        }
        let forward = transform.forward();
        let right = transform.right();
        transform.translation += options.velocity.x * dt * right
            + options.velocity.y * dt * Vec3::Y
            + options.velocity.z * dt * forward;

        // Handle mouse input
        let mut mouse_delta = Vec2::ZERO;
        if mouse_button_input.pressed(options.mouse_key_enable_mouse) || *move_toggled {
            for mut window in &mut windows {
                if !window.focused {
                    continue;
                }

                window.cursor.grab_mode = CursorGrabMode::Locked;
                window.cursor.visible = false;
            }

            for mouse_event in mouse_events.iter() {
                mouse_delta += mouse_event.delta;
            }
        }
        if mouse_button_input.just_released(options.mouse_key_enable_mouse) {
            for mut window in &mut windows {
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
            }
        }

        if mouse_delta != Vec2::ZERO {
            // Apply look update
            options.pitch = (options.pitch - mouse_delta.y * RADIANS_PER_DOT * options.sensitivity)
                .clamp(-PI / 2., PI / 2.);
            options.yaw -= mouse_delta.x * RADIANS_PER_DOT * options.sensitivity;
            transform.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, options.yaw, options.pitch);
        }
    }
}
