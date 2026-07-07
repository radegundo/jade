use bevy::{
  camera::{ RenderTarget, visibility::RenderLayers },
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
    .add_plugins(GridPlugin)
    .add_systems(Startup, setup)
    .add_systems(Startup, setup_gizmo_layers)
    .add_systems(Update, input::input)
    .add_systems(Update, ray::draw_rays)
    .add_systems(Update, map::draw_walls)
    // .add_systems(Update, check_rays)
    .insert_resource(Map {
      walls: vec![Wall::new(-100.0, -100.0, 100.0, 100.0), Wall::new(-100.0, 50.0, 100.0, 50.0)],
    })
    .init_gizmo_group::<MapGizmos>()
    .run();
}

//Player marker
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
  mut materials: ResMut<Assets<ColorMaterial>>,
  window_query: Query<&Window, With<PrimaryWindow>>
) {
  commands.spawn(Camera2d);
  if let Ok(window) = window_query.single() {
    let window_size = Vec2::new(window.width(), window.height());
    let mut grid = Grid::new(GridSize { x: 8, y: 8 });
    grid.build(window_size);
    commands.insert_resource(grid);
  }

  //Spawn Map Window
  let map_win = commands
    .spawn(Window { resolution: (500, 500).into(), resizable: false, ..default() })
    .id();

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
      ray_count: 100,
      max_distance: 1000.0,
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
struct MapGizmos;

fn setup_gizmo_layers(mut config_store: ResMut<GizmoConfigStore>) {
  let (config, _) = config_store.config_mut::<MapGizmos>();
  config.render_layers = RenderLayers::layer(1);
}
