use bevy::prelude::*;
use crate::{
    map::{ Map, MapGizmos, find_player_sector },
    ray::get_hit_sector,
    systems::get_relative_coords,
    *,
};
pub struct RelativeMapPlugin;

impl Plugin for RelativeMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_walls, draw_rays, draw_player));
    }
}

pub fn draw_walls(
    map: Res<Map>,
    mut gizmos: Gizmos<MapGizmos>,
    player_cache: Res<PlayerCameraCache>
) {
    let transform = &player_cache.transform;
    for sector in &map.sectors {
        for wall in &sector.walls {
            let start = get_relative_coords(transform, wall.start);
            let end = get_relative_coords(transform, wall.end);
            let color = if wall.back_side_def.is_some() {
                Color::srgb(0.35, 0.35, 0.35)
            } else {
                Color::WHITE
            };
            gizmos.line_2d(start, end, color);
        }
        for obstacle in &sector.obstacles {
            for edge in &obstacle.edges {
                let start = get_relative_coords(transform, edge.start);
                let end = get_relative_coords(transform, edge.end);
                gizmos.line_2d(start, end, Color::srgb(0.7, 0.5, 0.2));
            }
        }
    }
}

pub fn draw_rays(
    mut gizmos: Gizmos<MapGizmos>,
    player_cache: Res<PlayerCameraCache>,
    map: Res<Map>,
    view_info: Res<ViewInfo>
) {
    if let Some(sector) = find_player_sector(player_cache.transform.translation.truncate(), &map) {
        for i in 0..RAY_COUNT {
            if let Some(hit) = get_hit_sector(&player_cache.transform, &view_info, sector, &map, i) {
                let rel_hit_pos = get_relative_coords(&player_cache.transform, hit.pos);
                gizmos.line_2d(Vec2::ZERO, rel_hit_pos, Color::srgb(1.0, 0.0, 0.0));
            }
        }
    }
}

fn draw_player(mut gizmos: Gizmos<MapGizmos>) {
    gizmos.circle_2d(Isometry2d::default(), 5.0, Color::srgb(0.0, 1.0, 0.0));
    gizmos.line_2d(Vec2::ZERO, Vec2::new(0.0, 8.0), Color::srgb(0.0, 1.0, 0.0));
}
