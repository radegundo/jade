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
        get_sector_hits(&player_cache, &mut hits, &map.sectors[i], &view_info);
    } else {
        for hit in hits.hits.iter_mut() {
            *hit = None;
        }
    }
    for i in 0..RAY_COUNT {
        if let Some(hit) = &hits.hits[i] {
            if hit.line_def.back_side_def.is_none() {
                let x = hit_to_screen_x(&view_info, i);
                let wall_bottom = map.sectors[hit.sector_id].floor_height;
                let wall_top = wall_bottom + map.sectors[hit.sector_id].ceiling_height;
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
                render_portal_boundaries(
                    i,
                    &map.sectors[hit.line_def.front_side_def.sector],
                    &map.sectors[hit.line_def.back_side_def.clone().unwrap().sector],
                    &hit.line_def,
                    hit.perp_dist,
                    &mut gizmos,
                    &view_info
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
                let wall_bottom = map.sectors[hit.sector_id].floor_height;
                let wall_top = wall_bottom + map.sectors[hit.sector_id].ceiling_height;
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

pub fn update_eye_height(
    player_cache: Res<PlayerCameraCache>,
    map: Res<Map>,
    mut view_info: ResMut<ViewInfo>,
    time: Res<Time>
) {
    let pos = player_cache.transform.translation.truncate();

    if let Some(sector_idx) = find_player_sector(pos, &map) {
        let sector = &map.sectors[sector_idx];
        let target_eye_height = sector.floor_height + EYE_OFFSET;

        let speed = 8.0; // higher = snappier transition
        view_info.eye_height =
            view_info.eye_height +
            (target_eye_height - view_info.eye_height) * (speed * time.delta_secs()).min(1.0);
    }
}

pub fn render_portal_boundaries(
    index: usize,
    front_sector: &Sector,
    back_sector: &Sector,
    line_def: &LineDef,
    total_dist: f32,
    gizmos: &mut Gizmos,
    view_info: &ViewInfo
) {
    let x = hit_to_screen_x(view_info, index);
    if back_sector.ceiling_height < front_sector.ceiling_height {
        if let Some(color) = &line_def.front_side_def.upper_texture {
            let top = project_height(front_sector.ceiling_height, total_dist, view_info);
            let bottom = project_height(back_sector.ceiling_height, total_dist, view_info);
            gizmos.line_2d(Vec2::new(x, top), Vec2::new(x, bottom), *color);
        }
    }
    if back_sector.floor_height > front_sector.floor_height {
        if let Some(color) = &line_def.front_side_def.lower_texture {
            let top = project_height(back_sector.floor_height, total_dist, view_info);
            let bottom = project_height(front_sector.floor_height, total_dist, view_info);
            gizmos.line_2d(Vec2::new(x, top), Vec2::new(x, bottom), *color);
        }
    }
}

fn project_height(world_height: f32, dist: f32, view_info: &ViewInfo) -> f32 {
    let relative = world_height - view_info.eye_height;
    (relative * view_info.view_distance) / dist
}
