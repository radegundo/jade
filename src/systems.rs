use bevy::prelude::*;
use crate::*;

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
