use bevy::prelude::*;
use crate::*;
use map::*;

//--------------------------DATA STRUCTURES-------------------------------
pub struct Ray {
    start: Vec2,
    sec_point: Vec2,
}

#[derive(Clone)]
pub struct WallHit {
    pub pos: Vec2,
    pub perp_dist: f32,
    pub sector_id: usize,
    pub room_sector_id: usize,
    pub wall_id: WallId,
    pub is_portal: bool,
    pub back_sector: Option<usize>,
    pub ray_index: usize,
    pub bottom: f32,
    pub top: f32,
}

// ----------------------HELPER FUNCTIONS---------------------------
pub fn get_ray_angle(ray_index: usize, transform: &Transform, view_info: &ViewInfo) -> f32 {
    let player_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
    let fov_rad = view_info.fov.to_radians();
    let half_fov = fov_rad / 2.0;
    let angle_step = fov_rad / ((RAY_COUNT as f32) - 1.0).max(1.0);
    player_angle - half_fov + angle_step * (ray_index as f32)
}

pub fn get_ray_offset(ray_index: usize, view_info: &ViewInfo) -> f32 {
    let fov_rad = view_info.fov.to_radians();
    let half_fov = fov_rad / 2.0;
    let angle_step = fov_rad / ((RAY_COUNT as f32) - 1.0).max(1.0);
    -half_fov + angle_step * (ray_index as f32)
}

fn ray_hit(ray: &Ray, wall: &LineDef) -> Option<Vec2> {
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
        Some(Vec2::new(x1 + t * (x2 - x1), y1 + t * (y2 - y1)))
    } else {
        None
    }
}

//---------------------RAY HIT PER SECTOR---------------------------

pub fn get_hit_sector(
    transform: &Transform,
    view_info: &ViewInfo,
    sector_index: usize,
    map: &Map,
    index: usize
) -> Option<WallHit> {
    let origin = transform.translation.truncate();
    let angle = get_ray_angle(index, transform, view_info);
    let offset = get_ray_offset(index, view_info);
    let end = origin + Vec2::new(angle.cos(), angle.sin()) * view_info.max_distance;
    let ray = Ray { start: origin, sec_point: end };
    let sector = &map.sectors[sector_index];

    let mut nearest_hit: Option<(Vec2, WallId)> = None;
    let mut nearest_dist_sq = f32::MAX;

    for wall in &sector.walls {
        if let Some(hit) = ray_hit(&ray, wall) {
            let dist_sq = origin.distance_squared(hit);
            if dist_sq < nearest_dist_sq {
                nearest_dist_sq = dist_sq;
                nearest_hit = Some((hit, wall.id));
            }
        }
    }

    nearest_hit.map(|(pos, wall_id)| {
        let raw_dist = nearest_dist_sq.sqrt();
        let perp_dist = raw_dist * offset.cos();
        let wall = &sector.walls[wall_id.index];

        WallHit {
            pos,
            perp_dist,
            sector_id: wall_id.sector,
            room_sector_id: sector_index,
            wall_id,
            is_portal: wall.back_side_def.is_some(),
            back_sector: wall.back_side_def.as_ref().map(|s| s.facing),
            ray_index: index,
            bottom: sector.floor_height,
            top: sector.ceiling_height,
        }
    })
}

pub fn get_hit_sector_recursive(
    origin: Vec2,
    angle: f32,
    offset: f32,
    view_info: &ViewInfo,
    sector_index: usize,
    map: &Map,
    index: usize
) -> Option<WallHit> {
    let end = origin + Vec2::new(angle.cos(), angle.sin()) * view_info.max_distance;
    let ray = Ray { start: origin, sec_point: end };
    let sector = &map.sectors[sector_index];

    let mut nearest_hit: Option<(Vec2, WallId)> = None;
    let mut nearest_dist_sq = f32::MAX;

    for wall in &sector.walls {
        if let Some(hit) = ray_hit(&ray, wall) {
            let dist_sq = origin.distance_squared(hit);
            if dist_sq < nearest_dist_sq {
                nearest_dist_sq = dist_sq;
                nearest_hit = Some((hit, wall.id));
            }
        }
    }

    nearest_hit.map(|(pos, wall_id)| {
        let raw_dist = nearest_dist_sq.sqrt();
        let perp_dist = raw_dist * offset.cos();
        let wall = &sector.walls[wall_id.index];

        WallHit {
            pos,
            perp_dist,
            sector_id: wall_id.sector,
            room_sector_id: sector_index,
            wall_id,
            is_portal: wall.back_side_def.is_some(),
            back_sector: wall.back_side_def.as_ref().map(|s| s.facing),
            ray_index: index,
            bottom: sector.floor_height,
            top: sector.ceiling_height,
        }
    })
}

/// Tests if any ray from origin in this sector can see an obstacle.
/// Returns true if the ray intersects any obstacle edge closer than the wall.
pub fn ray_hits_obstacle(
    origin: Vec2,
    angle: f32,
    view_info: &ViewInfo,
    sector_index: usize,
    map: &Map,
    max_dist_sq: f32,
    obstacle_id: usize
) -> bool {
    let end = origin + Vec2::new(angle.cos(), angle.sin()) * view_info.max_distance;
    let ray = Ray { start: origin, sec_point: end };
    let sector = &map.sectors[sector_index];

    if let Some(obstacle) = sector.obstacles.iter().find(|o| o.id == obstacle_id) {
        for edge in &obstacle.edges {
            if let Some(hit_pos) = ray_hit(&ray, edge) {
                let dist_sq = origin.distance_squared(hit_pos);
                if dist_sq < max_dist_sq {
                    return true;
                }
            }
        }
    }

    false
}

//----------------------------SYSTEMS------------------

pub fn hit_to_screen_x(view_info: &ViewInfo, ray_index: usize) -> f32 {
    let angle = -get_ray_offset(ray_index, view_info);
    view_info.view_distance * angle.tan()
}

pub fn make_ray(start: Vec2, end: Vec2) -> Ray {
    Ray { start, sec_point: end }
}

pub fn test_ray_hit(ray: &Ray, wall: &LineDef, origin: Vec2, max_dist_sq: f32) -> bool {
    if let Some(hit_pos) = ray_hit(ray, wall) {
        let dist_sq = origin.distance_squared(hit_pos);
        dist_sq < max_dist_sq
    } else {
        false
    }
}
