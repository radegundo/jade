use bevy::prelude::*;
use crate::*;
use map::*;

//--------------------------DATA STRUCTURES-------------------------------

pub struct Ray {
    pub start: Vec2,
    pub sec_point: Vec2,
}

// Represents a single ray intersection with a sector wall.
// Carries everything the renderer needs to build a wall mesh slice.
#[derive(Clone)]
pub struct WallHit {
    // 2D world position where the ray hit the wall surface
    pub pos: Vec2,
    // Perpendicular distance from player — used for correct projection
    pub perp_dist: f32,
    // Which sector owns the wall that was hit
    pub sector_id: usize,
    // Which sector the ray was travelling through when it hit
    // Unique identifier for the specific wall that was hit
    pub wall_id: WallId,
    // True if this wall is a portal (has a back sector)
    pub is_portal: bool,
    // If portal, which sector is on the other side
    pub back_sector: Option<usize>,
    // Which ray index (0..RAY_COUNT) produced this hit
    pub ray_index: usize,
    // World Z of the bottom of this surface
    pub bottom: f32,
    // World Z of the top of this surface
    pub top: f32,
}

// ----------------------HELPER FUNCTIONS---------------------------

// Returns the absolute world-space angle for ray at index.
pub fn get_ray_angle(ray_index: usize, transform: &Transform, view_info: &ViewInfo) -> f32 {
    let player_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
    let fov_rad = view_info.fov.to_radians();
    let half_fov = fov_rad / 2.0;
    let angle_step = fov_rad / ((RAY_COUNT as f32) - 1.0).max(1.0);
    player_angle - half_fov + angle_step * (ray_index as f32)
}

// Returns the offset angle from the view center for ray at index.
// Used to correct fish-eye distortion via offset.cos().
pub fn get_ray_offset(ray_index: usize, view_info: &ViewInfo) -> f32 {
    let fov_rad = view_info.fov.to_radians();
    let half_fov = fov_rad / 2.0;
    let angle_step = fov_rad / ((RAY_COUNT as f32) - 1.0).max(1.0);
    -half_fov + angle_step * (ray_index as f32)
}

// Core 2D ray-segment intersection test.
// Returns the world position of intersection if the ray hits the segment,
// None otherwise.
fn ray_hit(ray: &Ray, start: Vec2, end: Vec2) -> Option<Vec2> {
    let (x1, y1) = (ray.start.x, ray.start.y);
    let (x2, y2) = (ray.sec_point.x, ray.sec_point.y);
    let (x3, y3) = (start.x, start.y);
    let (x4, y4) = (end.x, end.y);

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

// Exposes Ray construction for use in render.rs obstacle visibility testing.
pub fn make_ray(start: Vec2, end: Vec2) -> Ray {
    Ray { start, sec_point: end }
}

// Exposes single edge hit test for use in render.rs obstacle edge visibility.
// Returns true if the ray hits this wall closer than max_dist_sq.
pub fn test_ray_hit(
    ray: &Ray,
    wall: &LineDef,
    origin: Vec2,
    max_dist_sq: f32,
    vertices: &[Vec2]
) -> bool {
    if let Some(hit_pos) = ray_hit(ray, *wall.start(vertices), *wall.end(vertices)) {
        origin.distance_squared(hit_pos) < max_dist_sq
    } else {
        false
    }
}

//---------------------RAY HIT PER SECTOR---------------------------

// Used by render_2d (minimap). Fires one ray and returns the nearest wall hit
// in the given sector.
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
        let start = *wall.start(&map.vertices);
        let end = *wall.end(&map.vertices);
        if let Some(hit) = ray_hit(&ray, start, end) {
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
            wall_id,
            is_portal: wall.back_side_def.is_some(),
            back_sector: wall.back_side_def.as_ref().map(|s| s.facing),
            ray_index: index,
            bottom: sector.floor_height,
            top: sector.ceiling_height,
        }
    })
}

// Used by recurse_sector. Returns only the nearest wall hit for this sector.
// Obstacles are handled separately by ray_hits_obstacle.
pub fn get_hit_sector_recursive(
    origin: Vec2,
    dir: Vec2,
    offset: f32,
    view_info: &ViewInfo,
    sector_index: usize,
    map: &Map,
    index: usize
) -> Option<WallHit> {
    let end = origin + dir * view_info.max_distance;
    let ray = Ray { start: origin, sec_point: end };
    let sector = &map.sectors[sector_index];

    let mut nearest_hit: Option<(Vec2, WallId)> = None;
    let mut nearest_dist_sq = f32::MAX;

    for wall in &sector.walls {
        let start = *wall.start(&map.vertices);
        let end = *wall.end(&map.vertices);
        if let Some(hit) = ray_hit(&ray, start, end) {
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
            wall_id,
            is_portal: wall.back_side_def.is_some(),
            back_sector: wall.back_side_def.as_ref().map(|s| s.facing),
            ray_index: index,
            bottom: sector.floor_height,
            top: sector.ceiling_height,
        }
    })
}

// Returns true if any edge of the given obstacle is hit by this ray
// AND is closer than max_dist_sq (the nearest wall distance).
// Used to determine whether to show an obstacle's pre-spawned entities.
pub fn ray_hits_obstacle(
    origin: Vec2,
    end: Vec2,
    sector_index: usize,
    map: &Map,
    max_dist_sq: f32,
    obstacle_id: usize
) -> bool {
    let ray = Ray { start: origin, sec_point: end };
    let sector = &map.sectors[sector_index];

    if let Some(obstacle) = sector.obstacles.iter().find(|o| o.id == obstacle_id) {
        for edge in &obstacle.edges {
            let start = *edge.start(&map.vertices);
            let end = *edge.end(&map.vertices);
            if let Some(hit_pos) = ray_hit(&ray, start, end) {
                if origin.distance_squared(hit_pos) < max_dist_sq {
                    return true;
                }
            }
        }
    }
    false
}

//----------------------------SYSTEMS------------------

// Converts a ray index to its screen X position.
pub fn hit_to_screen_x(view_info: &ViewInfo, ray_index: usize) -> f32 {
    let angle = -get_ray_offset(ray_index, view_info);
    view_info.view_distance * angle.tan()
}
