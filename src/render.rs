use bevy::{ prelude::*, text::FontSource::Math, transform, window::PrimaryWindow };

use crate::*;
use ray::*;

pub fn render(
    mut gizmos: Gizmos,
    player_cache: Res<PlayerCameraCache>,
    hits: Res<Hits>,
    view_info: Res<ViewInfo>
) {
    for i in 0..RAY_COUNT {
        if let Some(hit) = hits.0[i] {
            let hitx = hit_to_screen_x(&view_info, &hits, i);
            println!("{:?}", hitx);
            gizmos.circle_2d(Isometry2d::from_xy(hitx.unwrap_or(0.0), 0.0), 10.0, Color::WHITE);
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
    let relX = dx * angle.cos() + dy * angle.sin();
    let relY = -dx * angle.sin() + dy * angle.cos();

    Vec2::new(relX, relY)
}
