use bevy::{
  camera::{ RenderTarget, visibility::RenderLayers },
  prelude::*,
  window::{ WindowRef, WindowResolution },
};
use bevy_grid::*;

use crate::map::*;
use render::*;

mod input;
mod ray;
mod map;
mod render;

const RAY_COUNT: usize = 100;
const WALL_HEIGHT: f32 = 20.0;

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
    .add_systems(Startup, setup)
    .add_systems(Startup, setup_gizmo_layers)
    .add_systems(Update, input::input)
    .add_systems(Update, map::draw_rays)
    .add_systems(Update, map::draw_walls)
    .add_systems(Update, draw_map_grid)
    .add_systems(Update, ray::get_hits)
    .add_systems(Update, render)
    .insert_resource(Map {
      walls: vec![Wall::new(-100.0, -100.0, 100.0, 100.0), Wall::new(-100.0, 50.0, 100.0, 50.0)],
    })
    .insert_resource(Hits::default())
    .init_gizmo_group::<MapGizmos>()
    .run();
}

//Player marker
#[derive(Component)]
struct Player;

#[derive(Component)]
struct FieldOfView {
  angle: f32,
  max_distance: f32,
}

#[derive(Resource)]
struct Hits([Option<Vec2>; RAY_COUNT]);

impl Default for Hits {
  fn default() -> Self {
    Hits([None; RAY_COUNT])
  }
}

fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>
) {
  commands.spawn(Camera2d);

  //Spawn Map Window
  let resolution: WindowResolution = (500, 500).into();
  let window_size = Vec2::new(resolution.width(), resolution.height());

  let map_win = commands
    .spawn((Window { resolution: resolution, resizable: false, ..default() }, MapWindowMarker))
    .id();

  let mut grid = Grid::new(GridSize { x: 8, y: 8 });
  grid.build(window_size);
  commands.insert_resource(grid);

  //Spawn Map Camera
  commands.spawn((
    Camera2d,
    RenderLayers::layer(1),
    RenderTarget::Window(WindowRef::Entity(map_win)),
  ));

  commands.insert_resource(MapWindow { id: map_win });

  //Spawn Map player
  commands.spawn((
    Player,
    FieldOfView {
      angle: 70.0,
      max_distance: 500.0,
    },
    Transform::default(),
    Mesh2d(meshes.add(Circle::new(10.0))),
    MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
    RenderLayers::layer(1),
    RenderTarget::Window(WindowRef::Entity(map_win)),
  ));
}

//Setup for different Gizmo configs
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct MapGizmos;

fn setup_gizmo_layers(mut config_store: ResMut<GizmoConfigStore>) {
  let (config, _) = config_store.config_mut::<MapGizmos>();
  config.render_layers = RenderLayers::layer(1);
}
