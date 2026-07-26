use bevy::prelude::*;

use crate::map::{ LineDef, Sector, WallId };

pub fn get_relative_coords(transform: &Transform, coords: Vec2) -> Vec2 {
    let dx = coords.x - transform.translation.x;
    let dy = coords.y - transform.translation.y;

    let angle = transform.rotation.to_euler(EulerRot::YXZ).0;
    let rel_x = dx * angle.cos() + dy * angle.sin();
    let rel_y = -dx * angle.sin() + dy * angle.cos();

    Vec2::new(rel_x, rel_y)
}

pub fn find_linedef_by_id(sector: &Sector, wall_id: WallId) -> LineDef {
    // Check sector walls first
    for wall in &sector.walls {
        if wall.id == wall_id {
            return wall.clone();
        }
    }
    // Check obstacle edges
    for obstacle in &sector.obstacles {
        for edge in &obstacle.edges {
            if edge.id == wall_id {
                return edge.clone();
            }
        }
    }
    // Fallback — should never happen if IDs are consistent
    panic!("LineDef with id {:?} not found in sector {}", wall_id, sector.id);
}
