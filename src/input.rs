use bevy::{ prelude::* };

use crate::{ Player, ViewInfo, map::MapViewMode };

pub struct OwnInputPlugin;

impl Plugin for OwnInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, input).add_systems(Update, toggle_map_view);
    }
}

pub fn input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
    mut view_info: ResMut<ViewInfo>
) {
    if let Ok(mut transform) = player_query.single_mut() {
        let angle = transform.rotation.to_euler(EulerRot::XYZ).2;
        let forward = Vec2::new(angle.cos(), angle.sin());
        let right = Vec2::new(-forward.y, forward.x); // perpendicular to forward

        let speed = 200.0; // units per second
        let mut movement = Vec2::ZERO;

        //MOVEMENT
        if keyboard_input.pressed(KeyCode::KeyW) {
            movement += forward;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            movement -= forward;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            movement += right;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            movement -= right;
        }

        if movement != Vec2::ZERO {
            movement = movement.normalize() * speed * time.delta_secs();
            transform.translation += movement.extend(0.0);
        }

        //TURNING
        let turn_speed = 5.0; // radians per second
        let mut rotation = 0.0;

        if keyboard_input.pressed(KeyCode::KeyQ) {
            rotation -= turn_speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyE) {
            rotation += turn_speed * time.delta_secs();
        }

        transform.rotate_z(rotation);

        if keyboard_input.pressed(KeyCode::KeyK) {
            view_info.pitch += 20.0;
        }
        if keyboard_input.pressed(KeyCode::KeyJ) {
            view_info.pitch -= 20.0;
        }
    }
}

fn toggle_map_view(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<MapViewMode>>,
    mut next_state: ResMut<NextState<MapViewMode>>
) {
    if keys.just_pressed(KeyCode::Tab) {
        next_state.set(match state.get() {
            MapViewMode::Relative => MapViewMode::Absolute,
            MapViewMode::Absolute => MapViewMode::Relative,
        });
    }
}
