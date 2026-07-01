use bevy::prelude::*;
use crate::*;

pub fn draw_ray(mut gizmos: Gizmos, query: Query<(&Transform, &Player, &FieldOfView)>) {
    if let Ok((transform, player, field_of_view)) = query.single() {
        for i in 0..field_of_view.ray_count {
            let angle =
                transform.rotation.to_euler(EulerRot::XYZ).2 +
                field_of_view.angle -
                field_of_view.angle / 2.0 +
                (field_of_view.angle / (field_of_view.ray_count as f32)) * (i as f32);
            let start = transform.translation;
            let end = start + Vec3::new(angle.cos(), angle.sin(), 0.0) * field_of_view.max_distance;
            gizmos.line(start, end, Color::srgb(1.0, 0.0, 0.0));
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct Ray {
    pub dir: f32,
    pub length: f32,
}
