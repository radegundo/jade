use bevy::prelude::*;
use crate::Player;

pub fn draw_ray(mut gizmos: Gizmos, query: Query<(&Transform, &Player)>) {
    if let Ok((transform, player)) = query.single() {
        for ray in player.rays.iter() {
            let start = transform.translation;
            let end = start + Vec3::new(ray.dir.cos(), ray.dir.sin(), 0.0) * ray.length;
            gizmos.line(start, end, Color::srgb(1.0, 0.0, 0.0));
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct Ray {
    pub dir: f32,
    pub length: f32,
}
