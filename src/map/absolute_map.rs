use bevy::prelude::*;

use crate::{ map::MapViewMode, * };
use ray::*;

pub struct AbsoluteMapPlugin;

impl Plugin for AbsoluteMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (draw_walls, draw_rays, draw_map_grid, draw_player).distributive_run_if(
                in_state(MapViewMode::Absolute)
            )
        );
    }
}

pub fn draw_walls(map: Res<Map>, mut gizmos: Gizmos<MapGizmos>) {
    for sector in &map.sectors {
        for wall in &sector.walls {
            gizmos.line(wall.start.extend(0.0), wall.end.extend(0.0), Color::WHITE);
        }
    }
}

pub fn draw_rays(
    mut gizmos: Gizmos<MapGizmos>,
    player_cache: Res<PlayerCameraCache>,
    hits: Res<Hits>,
    view_info: Res<ViewInfo>
) {
    let view_info = view_info.into_inner();
    let transform = &player_cache.transform;
    for i in 0..RAY_COUNT {
        // Get each ray's angle based on the player's rotation and the field of view
        let angle = get_ray_angle(i, transform, view_info);
        let start = transform.translation;
        let end = start + Vec3::new(angle.cos(), angle.sin(), 0.0) * view_info.max_distance;

        let draw_end = hits.hits[i]
            .as_ref()
            .map(|hit| hit.pos)
            .unwrap_or_else(|| end.truncate());

        // Draw to the nearest hit, or the full ray length if nothing was hit
        gizmos.line(start, draw_end.extend(0.0), Color::srgb(1.0, 0.0, 0.0));
    }
}

pub fn draw_map_grid(
    grid_o: Option<ResMut<Grid>>,
    mut gizmos: Gizmos<MapGizmos>,
    window_query: Query<&Window, With<MapWindowMarker>>
) {
    if let Some(mut grid) = grid_o {
        if let Ok(window) = window_query.single() {
            let window_size = Vec2::new(window.width(), window.height());
            grid.draw(&mut gizmos);
            grid.update_grid(window_size);
        }
    }
}

fn draw_player(player_cache: Res<PlayerCameraCache>, mut gizmos: Gizmos<MapGizmos>) {
    let transform = player_cache.transform.translation;
    gizmos.circle_2d(
        Isometry2d::from_xy(transform.x, transform.y),
        5.0,
        Color::srgb(1.0, 1.0, 1.0)
    );
}
