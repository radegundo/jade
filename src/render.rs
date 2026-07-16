use bevy::prelude::*;

use crate::*;
use ray::*;

pub fn render(
    mut gizmos: Gizmos,
    mut hits: ResMut<Hits>,
    view_info: Res<ViewInfo>,
    player_cache: Res<PlayerCameraCache>,
    map: Res<Map>,
    mut clip: ResMut<Vclip>
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
            clip.0[i] = VBounds::full();
            render_column(
                i,
                &player_cache,
                hit.perp_dist,
                hit,
                clip.0[i],
                &view_info,
                &mut gizmos,
                &map
            );
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

fn project_height(world_height: f32, dist: f32, view_info: &ViewInfo) -> f32 {
    let relative = world_height - view_info.eye_height;
    (relative * view_info.view_distance) / dist + view_info.pitch
}

pub fn render_column(
    index: usize,
    player_cache: &PlayerCameraCache,
    total_dist: f32,
    hit: &WallHit,
    mut clip: VBounds,
    view_info: &ViewInfo,
    gizmos: &mut Gizmos,
    map: &Map
) {
    let x = hit_to_screen_x(view_info, index);
    let sector = &map.sectors[hit.sector_id];

    let wall_top_screen = project_height(sector.ceiling_height, total_dist, view_info).clamp(
        clip.bottom,
        clip.top
    );
    let wall_bottom_screen = project_height(sector.floor_height, total_dist, view_info).clamp(
        clip.bottom,
        clip.top
    );

    match &hit.line_def.back_side_def {
        None => {
            // Solid wall —> draw within the clip window, then stop.
            gizmos.line_2d(
                Vec2::new(x, wall_top_screen),
                Vec2::new(x, wall_bottom_screen),
                hit.line_def.front_side_def.middle_texture.unwrap_or_default()
            );

            draw_floor(x, wall_bottom_screen, clip.bottom, sector.floor_texture, gizmos);
            draw_ceiling(x, wall_top_screen, clip.top, sector.ceiling_texture, gizmos);
        }
        Some(back) => {
            let back_sector = &map.sectors[back.sector];

            draw_floor(x, wall_bottom_screen, clip.bottom, sector.floor_texture, gizmos);
            draw_ceiling(x, wall_top_screen, clip.top, sector.ceiling_texture, gizmos);

            // Upper step (lowered ceiling ahead)
            if back_sector.ceiling_height < sector.ceiling_height {
                let upper_bottom = project_height(
                    back_sector.ceiling_height,
                    total_dist,
                    view_info
                ).clamp(clip.bottom, clip.top);
                if let Some(color) = hit.line_def.front_side_def.upper_texture {
                    gizmos.line_2d(
                        Vec2::new(x, wall_top_screen),
                        Vec2::new(x, upper_bottom),
                        color
                    );
                }
                clip.top = upper_bottom; // <-- SHRINK the window: nothing beyond can draw above this
            }

            // Lower step (raised floor ahead)
            if back_sector.floor_height > sector.floor_height {
                let lower_top = project_height(
                    back_sector.floor_height,
                    total_dist,
                    view_info
                ).clamp(clip.bottom, clip.top);
                if let Some(color) = hit.line_def.front_side_def.lower_texture {
                    gizmos.line_2d(
                        Vec2::new(x, lower_top),
                        Vec2::new(x, wall_bottom_screen),
                        color
                    );
                }
                clip.bottom = lower_top; // <-- SHRINK the window: nothing beyond can draw below this
            }
            // Lower floor ahead
            if back_sector.floor_height < sector.floor_height {
                let lower_top = project_height(sector.floor_height, total_dist, view_info).clamp(
                    clip.bottom,
                    clip.top
                );
                clip.bottom = lower_top;
            }
            if back_sector.ceiling_height > sector.ceiling_height {
                let upper_bottom = project_height(
                    sector.ceiling_height,
                    total_dist,
                    view_info
                ).clamp(clip.bottom, clip.top);
                clip.top = upper_bottom;
            }

            // Step into the next sector, carrying the SHRUNKEN clip forward
            let angle = get_ray_angle(index, &player_cache.transform, view_info);
            let dir = Vec2::new(angle.cos(), angle.sin());
            let nudged_origin = hit.pos + dir * 0.05;
            let mut nudged_transform = player_cache.transform.clone();
            nudged_transform.translation = nudged_origin.extend(0.0);

            if
                let Some(next_hit) = get_single_hit(
                    &nudged_transform,
                    view_info,
                    back_sector,
                    index
                )
            {
                let next_total_dist = total_dist + next_hit.perp_dist;
                render_column(
                    index,
                    player_cache,
                    next_total_dist,
                    &next_hit,
                    clip,
                    view_info,
                    gizmos,
                    map
                );
            }
        }
    }
}

fn draw_floor(x: f32, wall_bottom: f32, clip_bottom: f32, color: Color, gizmos: &mut Gizmos) {
    if wall_bottom > clip_bottom {
        gizmos.line_2d(Vec2::new(x, wall_bottom), Vec2::new(x, clip_bottom), color);
    }
}

fn draw_ceiling(x: f32, wall_top: f32, clip_top: f32, color: Color, gizmos: &mut Gizmos) {
    if wall_top < clip_top {
        gizmos.line_2d(Vec2::new(x, wall_top), Vec2::new(x, clip_top), color);
    }
}
