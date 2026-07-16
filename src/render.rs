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
            } else {
                render_portal(
                    i,
                    &player_cache,
                    hit.pos,
                    hit.perp_dist,
                    &map.sectors[hit.line_def.back_side_def.clone().unwrap().sector],
                    &view_info,
                    &mut gizmos,
                    &map
                );
            }
        }
    }
}

pub fn render_portal(
    index: usize,
    player_cache: &PlayerCameraCache,
    entry_pos: Vec2,
    entry_dist: f32,
    sector: &Sector,
    view_info: &ViewInfo,
    gizmos: &mut Gizmos,
    map: &Map
) {
    let angle = get_ray_angle(index, &player_cache.transform, view_info);
    let dir = Vec2::new(angle.cos(), angle.sin());
    let nudged_origin = entry_pos + dir * 0.01;

    let mut nudged_transform = player_cache.transform.clone();
    nudged_transform.translation = nudged_origin.extend(0.0);

    if let Some(hit) = get_single_hit(&nudged_transform, view_info, sector, index) {
        let total_dist = entry_dist + hit.perp_dist;

        match &hit.line_def.back_side_def {
            None => {
                // solid wall — draw it
                let x = hit_to_screen_x(view_info, index);
                let wall_bottom = 0.0;
                let wall_top = wall_bottom + WALL_HEIGHT;
                let top_relative = wall_top - view_info.eye_height;
                let bottom_relative = wall_bottom - view_info.eye_height;
                let top_screen = (top_relative * view_info.view_distance) / total_dist;
                let bottom_screen = (bottom_relative * view_info.view_distance) / total_dist;
                gizmos.line_2d(
                    Vec2::new(x, top_screen),
                    Vec2::new(x, bottom_screen),
                    hit.line_def.front_side_def.middle_texture.unwrap_or_default()
                );
            }
            Some(back) => {
                // another portal — keep going
                render_portal(
                    index,
                    player_cache,
                    hit.pos,
                    total_dist,
                    &map.sectors[back.sector],
                    view_info,
                    gizmos,
                    map
                );
            }
        }
    }
}
pub fn get_relative_coords(transform: &Transform, coords: Vec2) -> Vec2 {
    let dx = coords.x - transform.translation.x;
    let dy = coords.y - transform.translation.y;

    let angle = transform.rotation.to_euler(EulerRot::YXZ).0;
    let rel_x = dx * angle.cos() + dy * angle.sin();
    let rel_y = -dx * angle.sin() + dy * angle.cos();

    Vec2::new(rel_x, rel_y)
}
