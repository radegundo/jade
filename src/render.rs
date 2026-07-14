use bevy::prelude::*;

use crate::*;
use ray::*;

pub fn render(mut gizmos: Gizmos, hits: Res<Hits>, view_info: Res<ViewInfo>) {
    for i in 0..RAY_COUNT {
        if let Some(hit) = &hits.hits[i] {
            let x = hit_to_screen_x(&view_info, i);
            let wall_bottom = 0.0;
            let wall_top = wall_bottom + WALL_HEIGHT;
            let top_relative = wall_top - view_info.eye_height;
            let bottom_relative = wall_bottom - view_info.eye_height;

            let top_screen = (top_relative * view_info.view_distance) / hit.perp_dist;
            let bottom_screen = (bottom_relative * view_info.view_distance) / hit.perp_dist;

            gizmos.line_2d(Vec2::new(x, top_screen), Vec2::new(x, bottom_screen), Color::WHITE);
        }
    }
    // if let Ok((transform, view_info)) = query.single() && let Ok(window) = window_query.single() {
    // THROW OUT WALLS WITH Y < 0 -> Get relative coords
    // FIGURE OUT CLIPPING??
    // BSP?? -> Render per sector
    // GET WALL X COORDS ON SCREEN
    // IDEK MAN
    // }
}

pub fn get_relative_coords(transform: &Transform, coords: Vec2) -> Vec2 {
    let dx = coords.x - transform.translation.x;
    let dy = coords.y - transform.translation.y;

    let angle = transform.rotation.to_euler(EulerRot::YXZ).0;
    let rel_x = dx * angle.cos() + dy * angle.sin();
    let rel_y = -dx * angle.sin() + dy * angle.cos();

    Vec2::new(rel_x, rel_y)
}
