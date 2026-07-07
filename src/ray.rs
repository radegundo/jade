use bevy::prelude::*;
use crate::*;

pub fn get_ray_angle(
  ray_index: usize,
  transform: &Transform,
  field_of_view: &FieldOfView
) -> Option<f32> {
  let player_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
  let fov_rad = field_of_view.angle.to_radians();
  let half_fov = fov_rad / 2.0;

  // Angle between each ray, in radians
  let angle_step = fov_rad / ((field_of_view.ray_count as f32) - 1.0).max(1.0);
  let angle = player_angle - half_fov + angle_step * (ray_index as f32);
  return Some(angle);
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
