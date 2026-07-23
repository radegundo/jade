use bevy::prelude::*;
use crate::*;

//Only used for translating for ray_hit function for now
struct Ray {
    start: Vec2,
    //Sample a point in the direction of the ray
    sec_point: Vec2,
}

#[derive(Resource)]
pub struct WallHit {
    pub pos: Vec2,
    pub perp_dist: f32,
    pub sector_id: usize,
    pub room_sector_id: usize,
    pub line_def: LineDef,
    pub sector_type: SectorType,
}

#[derive(Resource)]
pub struct Hits {
    pub hits: Vec<Option<WallHit>>,
}

impl Hits {
    pub fn no_hits() -> Self {
        let mut hits = Vec::new();
        for _ in 0..RAY_COUNT {
            hits.push(None);
        }
        Hits { hits: hits }
    }
}

pub fn get_ray_angle(ray_index: usize, transform: &Transform, view_info: &ViewInfo) -> f32 {
    let player_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
    let fov_rad = view_info.fov.to_radians();
    let half_fov = fov_rad / 2.0;

    // Angle between each ray, in radians
    let angle_step = fov_rad / ((RAY_COUNT as f32) - 1.0).max(1.0);
    let angle = player_angle - half_fov + angle_step * (ray_index as f32);
    angle
}

pub fn get_ray_offset(ray_index: usize, view_info: &ViewInfo) -> f32 {
    let fov_rad = view_info.fov.to_radians();
    let half_fov = fov_rad / 2.0;
    let angle_step = fov_rad / ((RAY_COUNT as f32) - 1.0).max(1.0);
    // offset from center, NOT absolute world angle
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
        let hit_point = Vec2::new(x1 + t * (x2 - x1), y1 + t * (y2 - y1));
        return Some(hit_point);
    }
    return None;
}

pub fn get_sector_hits(
    player_cache: &PlayerCameraCache,
    hits: &mut Hits,
    sector_index: usize,
    map: &Map,
    view_info: &ViewInfo
) {
    for i in 0..RAY_COUNT {
        hits.hits[i] = get_single_hit(&player_cache.transform, view_info, sector_index, &map, i);
    }
}

// pub fn get_single_hit(
//     transform: &Transform,
//     view_info: &ViewInfo,
//     sector_index: usize,
//     map: &Map,
//     index: usize
// ) -> Option<WallHit> {
//     let origin = transform.translation.truncate();
//     let angle = get_ray_angle(index, &transform, view_info);
//     let offset = get_ray_offset(index, view_info); // needed for fisheye correction
//     let start = transform.translation;
//     let end = start + Vec3::new(angle.cos(), angle.sin(), 0.0) * view_info.max_distance;
//     let ray = Ray { start: start.truncate(), sec_point: end.truncate() };

//     let mut nearest_hit: Option<(Vec2, LineDef, SectorType)> = None;
//     let mut nearest_dist_sq = f32::MAX;

//     for wall in &map.sectors[sector_index].walls {
//         if let Some(hit) = ray_hit(&ray, wall) {
//             let dist_sq = origin.distance_squared(hit);
//             if dist_sq < nearest_dist_sq {
//                 nearest_dist_sq = dist_sq;
//                 nearest_hit = Some((hit, wall.clone(), SectorType::ObstacleSector));
//             }
//         }
//     }
//     if let Some(obstacle_ids) = &map.sectors[sector_index].obstacle_ids {
//         for obstacle in obstacle_ids {
//             for wall in &map.obstacle_sectors[*obstacle].walls {
//                 if let Some(hit) = ray_hit(&ray, wall) {
//                     let dist_sq = origin.distance_squared(hit);
//                     if dist_sq < nearest_dist_sq {
//                         nearest_dist_sq = dist_sq;
//                         nearest_hit = Some((hit, wall.clone(), SectorType::Sector));
//                     }
//                 }
//             }
//         }
//     }
//     if let Some(hit) = nearest_hit {
//         let raw_dist = nearest_dist_sq.sqrt(); // straight-line distance
//         let perp_dist = raw_dist * offset.cos(); // fisheye-corrected
//         Some(WallHit {
//             pos: hit.0,
//             perp_dist: perp_dist,
//             line_def: hit.1.clone(),
//             sector_id: hit.1.front_side_def.sector,
//             sector_type: hit.2,
//         })
//     } else {
//         None
//     }
// }
pub fn get_single_hit(
    transform: &Transform,
    view_info: &ViewInfo,
    sector_index: usize,
    map: &Map,
    index: usize
) -> Option<WallHit> {
    let origin = transform.translation.truncate();
    let angle = get_ray_angle(index, &transform, view_info);
    let offset = get_ray_offset(index, view_info);
    let start = transform.translation;
    let end = start + Vec3::new(angle.cos(), angle.sin(), 0.0) * view_info.max_distance;
    let ray = Ray { start: start.truncate(), sec_point: end.truncate() };

    // (pos, line_def, sector_type, resolved_id)
    let mut nearest_hit: Option<(Vec2, LineDef, SectorType, usize)> = None;
    let mut nearest_dist_sq = f32::MAX;

    for wall in &map.sectors[sector_index].walls {
        if let Some(hit) = ray_hit(&ray, wall) {
            let dist_sq = origin.distance_squared(hit);
            if dist_sq < nearest_dist_sq {
                nearest_dist_sq = dist_sq;
                nearest_hit = Some((hit, wall.clone(), SectorType::Sector, sector_index));
            }
        }
    }

    if let Some(obstacle_ids) = &map.sectors[sector_index].obstacle_ids {
        for &obstacle_id in obstacle_ids {
            for wall in &map.obstacle_sectors[obstacle_id].walls {
                if let Some(hit) = ray_hit(&ray, wall) {
                    let dist_sq = origin.distance_squared(hit);
                    if dist_sq < nearest_dist_sq {
                        nearest_dist_sq = dist_sq;
                        nearest_hit = Some((
                            hit,
                            wall.clone(),
                            SectorType::ObstacleSector,
                            obstacle_id,
                        ));
                    }
                }
            }
        }
    }

    nearest_hit.map(|(pos, line_def, sector_type, id)| {
        let raw_dist = nearest_dist_sq.sqrt();
        let perp_dist = raw_dist * offset.cos();
        WallHit {
            pos,
            perp_dist,
            sector_id: id, // now correct for both cases
            room_sector_id: sector_index,
            line_def,
            sector_type,
        }
    })
}

pub fn hit_to_screen_x(view_info: &ViewInfo, ray_index: usize) -> f32 {
    let angle = get_ray_offset(ray_index, &view_info);
    view_info.view_distance * angle.tan()
}

//Probably wont need this anymore
// pub fn perpendicular_distance(ray_hit_distance: f32, ray_offset: f32) -> f32 {
//     ray_hit_distance * ray_offset.cos()
// }
