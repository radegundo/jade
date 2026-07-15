use bevy::prelude::*;

use crate::*;
use ray::*;

pub fn render(
    mut gizmos: Gizmos,
    mut hits: ResMut<Hits>,
    view_info: Res<ViewInfo>,
    player_cache: Res<PlayerCameraCache>,
    map: Res<Map>
) {
    let player_pos = player_cache.transform.translation.truncate();
    if let Some(i) = find_player_sector(player_pos, &map) {
        println!("IN SECTOR: {i}");
        get_sector_hits(&player_cache, &mut hits, &map.sectors[i], &view_info);
    } else {
        println!("IN NO SECTOR");
        for hit in hits.hits.iter_mut() {
            *hit = None;
        }
    }
    for i in 0..RAY_COUNT {
        if let Some(hit) = &hits.hits[i] {
            if hit.line_def.back_side_def.is_none() {
                let x = hit_to_screen_x(&view_info, i);
                let wall_bottom = 0.0;
                let wall_top = wall_bottom + WALL_HEIGHT;
                let top_relative = wall_top - view_info.eye_height;
                let bottom_relative = wall_bottom - view_info.eye_height;

                let top_screen = (top_relative * view_info.view_distance) / hit.perp_dist;
                let bottom_screen = (bottom_relative * view_info.view_distance) / hit.perp_dist;

                gizmos.line_2d(
                    Vec2::new(x, top_screen),
                    Vec2::new(x, bottom_screen),
                    hit.line_def.front_side_def.middle_texture.unwrap_or_default()
                );
            }
        }
    }
    // if let Ok((transform, view_info)) = query.single() && let Ok(window) = window_query.single() {
    // THROW OUT WALLS WITH Y < 0 -> Get relative coords
    // FIGURE OUT CLIPPING??
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
