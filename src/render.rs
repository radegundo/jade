use bevy::prelude::*;

use crate::*;
use ray::*;

pub fn render(
  mut gizmos: Gizmos,
  query: Query<(&Transform, &FieldOfView), With<Player>>,
  map: Res<Map>
) {
  if let Ok((transform, field_of_view)) = query.single() {
    let origin = transform.translation.truncate();
    for i in 0..field_of_view.ray_count {
      // Get each ray's angle based on the player's rotation and the field of view
      let angle = get_ray_angle(i, transform, field_of_view);
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
      //MAKE RENDER LINES
    }
  }
}
