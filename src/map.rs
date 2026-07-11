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

pub struct MapPlugin;

impl Plugin for MapPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Update, draw_walls)
      .add_systems(Update, draw_rays)
      .add_systems(Update, draw_map_grid);
  }
}

impl Wall {
  pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
    Wall { start: Vec2::new(x0, y0), end: Vec2::new(x1, y1) }
  }
}

pub fn draw_walls(map: Res<Map>, mut gizmos: Gizmos<MapGizmos>) {
  for wall in &map.walls {
    gizmos.line(wall.start.extend(0.0), wall.end.extend(0.0), Color::srgb(1.0, 0.0, 0.0));
  }
}

pub fn draw_rays(
  mut gizmos: Gizmos<MapGizmos>,
  query: Query<(&Transform, &ViewInfo), With<Player>>,
  hits: Res<Hits>
) {
  if let Ok((transform, view_info)) = query.single() {
    for i in 0..RAY_COUNT {
      // Get each ray's angle based on the player's rotation and the field of view
      let angle = get_ray_angle(i, transform, view_info);
      let start = transform.translation;
      let end = start + Vec3::new(angle.cos(), angle.sin(), 0.0) * view_info.max_distance;
      let ray = Ray { start: start.truncate(), sec_point: end.truncate() };

      let draw_end = hits.0[i].unwrap_or(ray.sec_point);

      // Draw to the nearest hit, or the full ray length if nothing was hit
      gizmos.line(start, draw_end.extend(0.0), Color::srgb(1.0, 0.0, 0.0));
    }
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

#[derive(Resource)]
pub struct MapWindow {
  pub id: Entity,
}

#[derive(Component)]
pub struct MapWindowMarker;
