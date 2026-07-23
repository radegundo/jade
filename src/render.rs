use bevy::prelude::*;
use crate::*;
use map::*;
use ray::*;

pub fn render(
    mut gizmos: Gizmos<MapGizmos>,
    map: Res<Map>,
    view_info: Res<ViewInfo>,
    transform_query: Query<&Transform, With<Player>>
) {
    let transform = transform_query.single().unwrap();
    for i in 0..RAY_COUNT {
        let ray_angle = get_ray_angle(i, &transform, &view_info);
        let ray_offset = get_ray_offset(i, &view_info);
        let ray_dir = Vec2::new(ray_angle.cos(), ray_angle.sin());
        let ray_start = transform.translation.truncate();
        let ray_end = ray_start + ray_dir * view_info.max_distance;
    }
}
