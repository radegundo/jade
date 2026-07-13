use bevy::{
    camera::{ RenderTarget, visibility::RenderLayers },
    input::InputPlugin,
    prelude::*,
    window::{ WindowRef, WindowResolution },
};
use bevy_grid::*;

use crate::{
    input::OwnInputPlugin,
    map::{ absolute_map::AbsoluteMapPlugin, relative_map::RelativeMapPlugin, * },
};
use render::*;

mod input;
mod ray;
mod map;
mod render;

//Screen width
//For naming purposes duplicate the constant
const WINDOW_WIDTH: usize = 1920;
const WINDOW_HEIGHT: u32 = 1080;

const WALL_HEIGHT: f32 = 20.0;

const RAY_COUNT: usize = WINDOW_WIDTH;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "My Bevy App".to_string(),
                    resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT),
                    resizable: false,
                    ..default()
                }),
                ..default()
            })
        )
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_gizmo_layers)
        .add_plugins(OwnInputPlugin)
        .add_systems(Update, sync_player_camera)
        .add_plugins(AbsoluteMapPlugin)
        .add_plugins(RelativeMapPlugin)
        .init_state::<MapViewMode>()
        .add_systems(Update, ray::get_hits)
        .add_systems(Update, render)
        .insert_resource(Map {
            walls: vec![
                Wall::new(-100.0, -100.0, 100.0, 100.0),
                Wall::new(-100.0, 50.0, 100.0, 50.0),
                Wall::new(-200.0, -200.0, -200.0, 200.0),
                Wall::new(-200.0, 200.0, 200.0, 200.0),
                Wall::new(-200.0, -200.0, 200.0, -200.0),
                Wall::new(200.0, 200.0, 200.0, -200.0)
            ],
        })
        .insert_resource(ViewInfo::default())
        .insert_resource(Hits::default())
        .insert_resource(PlayerCameraCache::default())
        .init_gizmo_group::<MapGizmos>()
        .run();
}

//Player marker
#[derive(Component)]
struct Player;

#[derive(Resource)]
pub struct ViewInfo {
    pub fov: f32,
    pub max_distance: f32,
    //Distance which the screen sits from the players point of view
    pub view_distance: f32,
}

impl Default for ViewInfo {
    fn default() -> Self {
        let fov: f32 = 90.0;
        //Calculate the distance from the camera to the screen
        let view_distance = (WINDOW_WIDTH as f32) / 2.0 / (fov / 2.0).tan();
        ViewInfo { fov, max_distance: 500.0, view_distance }
    }
}

#[derive(Resource)]
struct Hits([Option<Vec2>; RAY_COUNT]);

impl Default for Hits {
    fn default() -> Self {
        Hits([None; RAY_COUNT])
    }
}

//Resource that syncs the player position to a resource
#[derive(Resource, Default)]
pub struct PlayerCameraCache {
    transform: Transform,
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
    commands.spawn((Player, Transform::default()));
}

//Setup for different Gizmo configs
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct MapGizmos;

fn setup_gizmo_layers(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<MapGizmos>();
    config.render_layers = RenderLayers::layer(1);
}

//Sync the players position and angle to a resource
fn sync_player_camera(
    query: Query<&Transform, With<Player>>,
    mut cache: ResMut<PlayerCameraCache>
) {
    if let Ok(transform) = query.single() {
        cache.transform = *transform;
    }
}
