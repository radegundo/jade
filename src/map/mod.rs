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

#[derive(Default)]
pub struct LineDef {
    pub start: Vec2,
    pub end: Vec2,
    pub front_side_def: SideDef,
    pub back_side_def: Option<SideDef>,
}

#[derive(Default)]
pub struct SideDef {
    pub upper_texture: Option<Color>,
    pub middle_texture: Option<Color>,
    pub lower_texture: Option<Color>,
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

impl LineDef {
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32, color: Color) -> Self {
        let front_side_def = SideDef { middle_texture: Some(color), ..default() };
        LineDef { start: Vec2::new(x0, y0), end: Vec2::new(x1, y1), front_side_def, ..default() }
    }
}
