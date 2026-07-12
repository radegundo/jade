use bevy::{ prelude::*, text::FontSource::Math, transform, window::PrimaryWindow };

use crate::*;
use ray::*;

pub fn render(
  mut gizmos: Gizmos,
  query: Query<(&Transform, &ViewInfo), With<Player>>,
  window_query: Query<&Window, With<PrimaryWindow>>,
  hits: Res<Hits>
) {
  // if let Ok((transform, view_info)) = query.single() && let Ok(window) = window_query.single() {
  //   let origin = transform.translation.truncate();
  //   let window_size = window.size();
  //   for i in 0..RAY_COUNT {
  //     let line_width = window_size.x / (RAY_COUNT as f32);

  //     let iso = Isometry2d::from_xy(
  //       line_width * ((i as f32) - (RAY_COUNT as f32) / 2.0) + line_width / 2.0,
  //       0.0
  //     );
  //     let player_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
  //     let ray_angle = get_ray_angle(i, transform, view_info);
  //     let relative_angle = ray_angle - player_angle; // offset from center of FOV

  //     let mut line_height: f32 = 0.0;
  //     if let Some(hit) = hits.0[i] {
  //       let dist = origin.distance(hit) * relative_angle.cos();
  //       if dist <= view_info.max_distance {
  //         line_height = (WALL_HEIGHT * window_size.y) / dist;
  //       }
  //     }
  //     gizmos.rect_2d(iso, Vec2::new(line_width, line_height), Color::srgb(1.0, 0.0, 0.0));
  //     //MAKE RENDER LINES
  //   }
  // }
  if let Ok((transform, view_info)) = query.single() && let Ok(window) = window_query.single() {
    // THROW OUT WALLS WITH Y < 0 -> Get relative coords
    // FIGURE OUT CLIPPING??
    // BSP?? -> Render per sector
    // GET WALL X COORDS ON SCREEN
    // IDEK MAN
  }
}

pub fn get_relative_coords(transform: &Transform, coords: Vec2) -> Vec2 {
  let dx = coords.x - transform.translation.x;
  let dy = coords.y - transform.translation.y;

  let angle = transform.rotation.to_euler(EulerRot::YXZ).0;
  let relX = dx * angle.cos() + dy * angle.sin();
  let relY = -dx * angle.sin() + dy * angle.cos();

  Vec2::new(relX, relY)
}
