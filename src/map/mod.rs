pub mod absolute_map;
pub mod relative_map;

use bevy::prelude::*;

#[derive(Resource)]
pub struct Map {
  pub walls: Vec<Wall>,
}

#[derive(Default)]
pub struct Wall {
  pub start: Vec2,
  pub end: Vec2,
  pub front_side_def: SideDef,
  pub back_side_def: Option<SideDef>,
}

#[derive(Default)]
pub struct SideDef {
  upper_texture: Option<Color>,
  middle_texture: Option<Color>,
  lower_texture: Option<Color>,
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
