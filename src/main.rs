use bevy::{
  camera::RenderTarget,
  prelude::*,
  window::{ PrimaryWindow, WindowRef, WindowResolution },
};
use bevy_grid::{ self, Grid, GridPlugin, GridSize };

use crate::map::*;

mod input;
mod ray;
mod map;

fn main() {
  App::new()
    .add_plugins(
      DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
          title: "My Bevy App".to_string(),
          resolution: WindowResolution::new(500, 500),
          resizable: false,
          ..default()
        }),
        ..default()
      })
    )
    // .add_plugins(GridPlugin)
    .add_systems(Startup, setup)
    .add_systems(Startup, map_window)
    // .add_systems(Update, input::input)
    // .add_systems(Update, ray::draw_rays)
    // .add_systems(Update, map::draw_walls)
    // .add_systems(Update, check_rays)
    .insert_resource(Map {
      walls: vec![Wall::new(-100.0, -100.0, 100.0, 100.0), Wall::new(-100.0, 50.0, 100.0, 50.0)],
    })
    .run();
}

#[derive(Component)]
struct Player;
#[derive(Component)]
struct FieldOfView {
  angle: f32,
  ray_count: usize,
  max_distance: f32,
}

#[derive(Resource)]
struct MapWindow {
  id: Entity,
}

fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>
) {
  //Spawn player
  commands.spawn((
    Player,
    FieldOfView {
      angle: 70.0,
      ray_count: 100,
      max_distance: 1000.0,
    },
    Transform::default(),
    Mesh2d(meshes.add(Circle::new(10.0))),
    MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
  ));
}

fn map_window(mut commands: Commands) {
  let second_window = commands
    .spawn(Window {
      title: "My Bevy App".into(),
      resolution: (500, 500).into(),
      ..default()
    })
    .id();
  commands.insert_resource(second_window);
}
