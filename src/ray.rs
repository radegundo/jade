use bevy::prelude::*;
use crate::*;

pub fn draw_ray(mut gizmos: Gizmos, query: Query<(&Transform, &FieldOfView), With<Player>>) {
    if let Ok((transform, field_of_view)) = query.single() {
        for i in 0..field_of_view.ray_count {
            //Get each ray's angle based on the player's rotation and the field of view
            let angle =
                //Player's angle
                transform.rotation.to_euler(EulerRot::XYZ).2 +
                //Total fov angle - half to get the starting angle
                field_of_view.angle -
                field_of_view.angle / 2.0 +
                //Angle between each ray
                (field_of_view.angle / (field_of_view.ray_count as f32)) * (i as f32);
            let start = transform.translation;
            let end = start + Vec3::new(angle.cos(), angle.sin(), 0.0) * field_of_view.max_distance;
            gizmos.line(start, end, Color::srgb(1.0, 0.0, 0.0));
        }
    }
}

// #[derive(Component, Clone, Debug)]
// pub struct Ray {
//     pub dir: f32,
//     pub length: f32,
// }
