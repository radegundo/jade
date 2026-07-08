use std::mem::transmute;

use bevy::{ prelude::*, window::PrimaryWindow };

use crate::*;
use ray::*;

pub fn render(
  mut gizmos: Gizmos,
  query: Query<(&Transform, &FieldOfView), With<Player>>,
  window_query: Query<&Window, With<PrimaryWindow>>,
  hits: Res<Hits>
) {
  if let Ok((transform, field_of_view)) = query.single() && let Ok(window) = window_query.single() {
    let origin = transform.translation.truncate();
    let window_size = window.size();
    for i in 0..RAY_COUNT {
      let line_width = window_size.x / (RAY_COUNT as f32);
      let iso = Isometry2d::from_xy(
        line_width * ((i as f32) - (RAY_COUNT as f32) / 2.0) + line_width / 2.0,
        0.0
      );
      let mut line_height: f32 = 0.0;
      if let Some(hit) = hits.0[i] {
        let dist = origin.distance(hit);
        if dist <= field_of_view.max_distance {
          line_height = (WALL_HEIGHT * window_size.y) / dist;
        }
      }
      gizmos.rect_2d(iso, Vec2::new(line_width, line_height), Color::srgb(1.0, 0.0, 0.0));
      //MAKE RENDER LINES
    }
  }
}
