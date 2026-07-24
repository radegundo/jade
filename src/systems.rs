use bevy::prelude::*;
use crate::*;

#[derive(Resource)]
pub struct WallEntityPool {
    pub entities: Vec<Entity>,
    pub used: usize,
}

pub fn get_relative_coords(transform: &Transform, coords: Vec2) -> Vec2 {
    let dx = coords.x - transform.translation.x;
    let dy = coords.y - transform.translation.y;

    let angle = transform.rotation.to_euler(EulerRot::YXZ).0;
    let rel_x = dx * angle.cos() + dy * angle.sin();
    let rel_y = -dx * angle.sin() + dy * angle.cos();

    Vec2::new(rel_x, rel_y)
}

pub fn point_in_sector(point: Vec2, sector: &Sector) -> bool {
    let mut inside = false;
    for wall in &sector.walls {
        let (x1, y1) = (wall.start.x, wall.start.y);
        let (x2, y2) = (wall.end.x, wall.end.y);
        let crosses = (y1 > point.y) != (y2 > point.y);
        if crosses {
            let x_intersect = x1 + ((point.y - y1) / (y2 - y1)) * (x2 - x1);
            if point.x < x_intersect {
                inside = !inside;
            }
        }
    }
    inside
}

pub fn find_player_sector(player_pos: Vec2, map: &Map) -> Option<usize> {
    for (i, sector) in map.sectors.iter().enumerate() {
        if point_in_sector(player_pos, sector) {
            return Some(i);
        }
    }
    None
}

pub fn hit_to_screen_x(view_info: &ViewInfo, ray_index: usize) -> f32 {
    let angle = -get_ray_offset(ray_index, &view_info);
    view_info.view_distance * angle.tan()
}

pub fn group_hits_by_wall(hits: Vec<WallHit>) -> Vec<Vec<WallHit>> {
    let mut grouped_hits: Vec<Vec<WallHit>> = Vec::new();
    let mut current_group: Vec<WallHit> = Vec::new();

    for hit in hits {
        if current_group.is_empty() {
            current_group.push(hit);
        } else {
            let last_hit = current_group.last().unwrap();
            if last_hit.wall_id == hit.wall_id && last_hit.sector_id == hit.sector_id {
                current_group.push(hit);
            } else {
                grouped_hits.push(current_group);
                current_group = vec![hit];
            }
        }
    }

    if !current_group.is_empty() {
        grouped_hits.push(current_group);
    }

    grouped_hits
}
