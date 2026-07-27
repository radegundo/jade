use bevy::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::window::{ CursorGrabMode, CursorOptions, PrimaryWindow };

use crate::{ Player, ViewInfo };

const YAW_SENSITIVITY: f32 = 0.002;
const PITCH_SENSITIVITY: f32 = 0.002;

pub struct OwnInputPlugin;

impl Plugin for OwnInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, grab_cursor)
            .add_systems(Update, input)
            .add_systems(Update, mouse_look);
    }
}

fn grab_cursor(mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor_options.visible = false;
    cursor_options.grab_mode = CursorGrabMode::Locked;
}

pub fn input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>
) {
    if let Ok(mut transform) = player_query.single_mut() {
        let angle = transform.rotation.to_euler(EulerRot::XYZ).2;
        let forward = Vec2::new(angle.cos(), angle.sin());
        let right = Vec2::new(-forward.y, forward.x);

        let speed = 100.0;
        let mut movement = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) {
            movement += forward;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            movement -= forward;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            movement -= right;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            movement += right;
        }

        if movement != Vec2::ZERO {
            movement = movement.normalize() * speed * time.delta_secs();
            transform.translation += movement.extend(0.0);
        }
    }
}

fn mouse_look(
    mut player_query: Query<&mut Transform, With<Player>>,
    accumulated: Res<AccumulatedMouseMotion>,
    mut view_info: ResMut<ViewInfo>
) {
    let delta = accumulated.delta;

    if delta == Vec2::ZERO {
        return;
    }

    if let Ok(mut transform) = player_query.single_mut() {
        let yaw = -delta.x * YAW_SENSITIVITY;
        transform.rotate_z(yaw);
    }

    view_info.pitch += -delta.y * PITCH_SENSITIVITY;
    view_info.pitch = view_info.pitch.clamp(-2.0, 2.0);
}
