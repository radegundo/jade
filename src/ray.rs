use bevy::prelude::*;
use crate::*;

pub fn get_ray_angle(ray_index: usize, transform: &Transform, field_of_view: &FieldOfView) -> f32 {
  let player_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
  let fov_rad = field_of_view.angle.to_radians();
  let half_fov = fov_rad / 2.0;

  // Angle between each ray, in radians
  let angle_step = fov_rad / ((RAY_COUNT as f32) - 1.0).max(1.0);
  let angle = player_angle - half_fov + angle_step * (ray_index as f32);
  angle
}

pub struct Ray {
  pub start: Vec2,
  //Sample a point in the direction of the ray
  pub sec_point: Vec2,
}

pub fn ray_hit(ray: &Ray, wall: &Wall) -> Option<Vec2> {
  let (x1, y1) = (ray.start.x, ray.start.y);
  let (x2, y2) = (ray.sec_point.x, ray.sec_point.y);
  let (x3, y3) = (wall.start.x, wall.start.y);
  let (x4, y4) = (wall.end.x, wall.end.y);

  let denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
  if denom == 0.0 {
    return None;
  }
  let t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom;
  let u = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / denom;

  if t >= 0.0 && u >= 0.0 && u <= 1.0 {
    let hit_point = Vec2::new(x1 + t * (x2 - x1), y1 + t * (y2 - y1));
    return Some(hit_point);
  }
  return None;
}

pub fn get_hits(
  query: Query<(&Transform, &FieldOfView), With<Player>>,
  mut hits: ResMut<Hits>,
  map: Res<Map>
) {
  if let Ok((transform, field_of_view)) = query.single() {
    let origin = transform.translation.truncate();
    for i in 0..RAY_COUNT {
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
      if let Some(_) = nearest_hit {
        hits.0[i] = nearest_hit;
      } else {
        hits.0[i] = None;
      }
    }
  }
}
