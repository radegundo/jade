use bevy::prelude::*;

use crate::Player;

pub fn input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut Player)>
) {
    if let Ok((mut transform, mut player)) = player_query.single_mut() {
        let mut dir = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::KeyD) {
            dir.x += 10.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            dir.x -= 10.0;
        }
        if keyboard_input.pressed(KeyCode::KeyW) {
            dir.y += 10.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            dir.y -= 10.0;
        }
        transform.translation += dir;

        if keyboard_input.pressed(KeyCode::KeyE) {
            for ray in player.rays.iter_mut() {
                ray.dir -= 0.1;
            }
        }
        if keyboard_input.pressed(KeyCode::KeyQ) {
            for ray in player.rays.iter_mut() {
                ray.dir += 0.1;
            }
        }
    }
}
