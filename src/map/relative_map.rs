use bevy::prelude::*;
use crate::*;
use systems::*;
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
    for sector in &map.sectors {
        for wall in &sector.walls {
            let transform = &player_cache.transform;
            let start: Vec2 = get_relative_coords(transform, wall.start);
            let end: Vec2 = get_relative_coords(transform, wall.end);
            gizmos.line(start.extend(0.0), end.extend(0.0), Color::WHITE);
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
            if let Some(hit) = get_single_hit(&player_cache.transform, &view_info, sector, &map, i) {
                let rel_hit_pos = get_relative_coords(&player_cache.transform, hit.pos);
                gizmos.line_2d(Vec2::ZERO, rel_hit_pos, Color::srgb(1.0, 0.0, 0.0));
            }
        }
    }
}

fn draw_player(mut gizmos: Gizmos<MapGizmos>) {
    gizmos.circle_2d(Isometry2d::default(), 5.0, Color::srgb(1.0, 1.0, 1.0));
}
