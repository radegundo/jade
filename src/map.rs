use bevy::prelude::*;

use crate::*;
use ray::*;

#[derive(Resource)]
pub struct Map {
  pub walls: Vec<Wall>,
}

pub struct Wall {
  pub start: Vec2,
  pub end: Vec2,
}

impl Wall {
  pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
    Wall { start: Vec2::new(x0, y0), end: Vec2::new(x1, y1) }
  }
}

pub fn draw_walls(map: Res<Map>, mut gizmos: Gizmos<MapGizmos>) {
  for wall in &map.walls {
    gizmos.line(wall.start.extend(0.0), wall.end.extend(0.0), Color::WHITE);
  }
}

pub fn draw_rays(
  mut gizmos: Gizmos<MapGizmos>,
  query: Query<(&Transform, &FieldOfView), With<Player>>,
  map: Res<Map>
) {
  if let Ok((transform, field_of_view)) = query.single() {
    let origin = transform.translation.truncate();

    for i in 0..field_of_view.ray_count {
      // Get each ray's angle based on the player's rotation and the field of view
      if let Some(angle) = get_ray_angle(i, transform, field_of_view) {
        let start = transform.translation;
        let end = start + Vec3::new(angle.cos(), angle.sin(), 0.0) * field_of_view.max_distance;
        let ray = Ray { start: start.truncate(), sec_point: end.truncate() };

        let mut nearest_hit: Option<Vec2> = None;
        let mut nearest_dist_sq = f32::MAX;

        for wall in &map.walls {
          if let Some(hit) = ray_hit(&ray, wall) {
            let dist_sq = origin.distance_squared(hit);
            if dist_sq < nearest_dist_sq {
              nearest_dist_sq = dist_sq;
              nearest_hit = Some(hit);
            }
          }
        }

        // Draw to the nearest hit, or the full ray length if nothing was hit
        let draw_end = nearest_hit.unwrap_or(ray.sec_point);
        gizmos.line(start, draw_end.extend(0.0), Color::srgb(1.0, 0.0, 0.0));
      }
    }
  }
}

pub fn draw_map_grid(
  mut grid_o: Option<ResMut<Grid>>,
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

#[derive(Resource)]
pub struct MapWindow {
  pub id: Entity,
}

#[derive(Component)]
pub struct MapWindowMarker;
