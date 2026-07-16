pub mod absolute_map;
pub mod relative_map;

use bevy::prelude::*;

#[derive(Resource)]
pub struct Sector {
    pub walls: Vec<LineDef>,
}

#[derive(Resource)]
pub struct Map {
    pub sectors: Vec<Sector>,
}

#[derive(Default, Clone)]
pub struct LineDef {
    pub start: Vec2,
    pub end: Vec2,
    pub front_side_def: SideDef,
    pub back_side_def: Option<SideDef>,
}

#[derive(Default, Clone)]
pub struct SideDef {
    pub upper_texture: Option<Color>,
    pub middle_texture: Option<Color>,
    pub lower_texture: Option<Color>,
    pub sector: usize,
}

#[derive(Resource)]
pub struct MapWindow {
    pub id: Entity,
}

#[derive(Component)]
pub struct MapWindowMarker;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum MapViewMode {
    #[default]
    Relative,
    Absolute,
}

pub fn portal(x0: f32, y0: f32, x1: f32, y1: f32, back_sector: usize) -> LineDef {
    LineDef {
        start: Vec2::new(x0, y0),
        end: Vec2::new(x1, y1),
        front_side_def: SideDef {
            middle_texture: None, // no middle texture -> see-through
            ..default()
        },
        back_side_def: Some(SideDef {
            middle_texture: None,
            sector: back_sector,
            ..Default::default()
        }),
    }
}

impl LineDef {
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32, color: Color) -> Self {
        let front_side_def = SideDef { middle_texture: Some(color), ..default() };
        LineDef {
            start: Vec2::new(x0, y0),
            end: Vec2::new(x1, y1),
            front_side_def,
            back_side_def: None,
        }
    }
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
