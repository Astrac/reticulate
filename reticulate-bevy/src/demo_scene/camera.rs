use bevy::{
    camera_controller::free_camera::{
        FreeCamera, FreeCameraPlugin, FreeCameraState, VerticalMovementAxis,
    },
    prelude::*,
};

const OVERLAY_FONT_SIZE: f32 = 13.0;

#[derive(Clone, Component, Copy)]
struct CameraOverlay;

#[derive(Clone, Copy, Resource)]
struct CameraOverlayShowing(bool);

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FreeCameraPlugin)
            .insert_resource(CameraOverlayShowing(true))
            .add_systems(Startup, spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(4.0, 7.0, -12.0).looking_at(Vec3::ZERO, Vec3::Y),
        FreeCamera {
            sensitivity: 0.2,
            friction: 25.0,
            walk_speed: 3.0,
            run_speed: 9.0,
            ..default()
        },
    ));
}

pub struct CameraSettingsPlugin;
impl Plugin for CameraSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_text_system)
            .add_systems(Update, (update_camera_settings, update_text));
    }
}

#[derive(Component)]
pub struct InfoText;

fn spawn_text_system(commands: Commands, free_camera: Query<&FreeCamera>) {
    spawn_text(commands, free_camera.single().unwrap());
}

fn spawn_text(mut commands: Commands, free_camera: &FreeCamera) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(-16),
            left: px(12),
            ..default()
        },
        children![(
            Text::new(format!("{}", free_camera)),
            TextFont::default().with_font_size(OVERLAY_FONT_SIZE),
        )],
        CameraOverlay,
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(8),
            left: px(12),
            ..default()
        },
        children![(
            Text::new(concat![
                "Z/X: decrease/increase sensitivity\n",
                "C/V: decrease/increase friction\n",
                "F/G: decrease/increase scroll factor\n",
                "B: enable/disable controller\n",
                "T: world/local vertical movement\n",
                "Enter: Next demo\n",
                "Tab: Loop triangles/shaded views\n",
                "/: Enable/Disable camera overlay",
            ]),
            TextFont::default().with_font_size(OVERLAY_FONT_SIZE),
        )],
        CameraOverlay,
    ));

    // Mutable text marked with component
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            right: px(12),
            ..default()
        },
        children![(
            InfoText,
            Text::new(""),
            TextFont::default().with_font_size(OVERLAY_FONT_SIZE),
        )],
        CameraOverlay,
    ));
}

fn update_camera_settings(
    mut commands: Commands,
    mut camera_query: Query<(&mut FreeCamera, &mut FreeCameraState)>,
    overlay_query: Query<Entity, With<CameraOverlay>>,
    mut overlay_showing: ResMut<CameraOverlayShowing>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let (mut free_camera, mut free_camera_state) = camera_query.single_mut().unwrap();

    if input.pressed(KeyCode::KeyZ) {
        free_camera.sensitivity = (free_camera.sensitivity - 0.005).max(0.005);
    }
    if input.pressed(KeyCode::KeyX) {
        free_camera.sensitivity += 0.005;
    }
    if input.pressed(KeyCode::KeyC) {
        free_camera.friction = (free_camera.friction - 0.2).max(0.0);
    }
    if input.pressed(KeyCode::KeyV) {
        free_camera.friction += 0.2;
    }
    if input.pressed(KeyCode::KeyF) {
        free_camera.scroll_factor = (free_camera.scroll_factor - 0.02).max(0.02);
    }
    if input.pressed(KeyCode::KeyG) {
        free_camera.scroll_factor += 0.02;
    }
    if input.just_pressed(KeyCode::KeyB) {
        free_camera_state.enabled = !free_camera_state.enabled;
    }
    if input.just_pressed(KeyCode::KeyT) {
        free_camera.vertical_movement_axis = match free_camera.vertical_movement_axis {
            VerticalMovementAxis::World => VerticalMovementAxis::Local,
            VerticalMovementAxis::Local => VerticalMovementAxis::World,
        };
    }
    if input.just_pressed(KeyCode::Slash) {
        if overlay_showing.0 {
            for o in overlay_query {
                commands.entity(o).despawn();
            }
        } else {
            spawn_text(commands, camera_query.single().unwrap().0);
        }

        overlay_showing.0 = !overlay_showing.0;
    }
}

fn update_text(
    mut text_query: Query<&mut Text, With<InfoText>>,
    camera_query: Query<(&FreeCamera, &FreeCameraState)>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        let (free_camera, free_camera_state) = camera_query.single().unwrap();

        text.0 = format!(
            "Enabled: {},\nSensitivity: {:.03}\nFriction: {:.01}\nScroll factor: {:.02}\nWalk Speed: {:.02}\nRun Speed: {:.02}\nSpeed: {:.02}",
            free_camera_state.enabled,
            free_camera.sensitivity,
            free_camera.friction,
            free_camera.scroll_factor,
            free_camera.walk_speed * free_camera_state.speed_multiplier,
            free_camera.run_speed * free_camera_state.speed_multiplier,
            free_camera_state.velocity.length(),
        );
    }
}
