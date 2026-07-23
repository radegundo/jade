use bevy::{
    camera::{ RenderTarget, visibility::RenderLayers },
    prelude::*,
    window::{ PresentMode, WindowRef, WindowResolution },
};
use bevy_grid::*;

use crate::{
    input::OwnInputPlugin,
    map::{ absolute_map::AbsoluteMapPlugin, relative_map::RelativeMapPlugin, * },
    ray::Hits,
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

const EYE_OFFSET: f32 = 1.6;

const RAY_COUNT: usize = WINDOW_WIDTH;

const FLOOR_NORMAL: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const CEILING_NORMAL: Vec3 = Vec3::new(0.0, 0.0, -1.0);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "My Bevy App".to_string(),
                    resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT),
                    present_mode: PresentMode::AutoVsync,
                    resizable: false,
                    ..default()
                }),
                ..default()
            })
        )
        //SETUP
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_gizmo_layers)
        //INPUT
        .add_plugins(OwnInputPlugin)
        //SYNC
        .add_systems(Update, sync_player_camera)
        .add_systems(Update, update_eye_height)
        //MAP
        .add_plugins(AbsoluteMapPlugin)
        .add_plugins(RelativeMapPlugin)
        .init_state::<MapViewMode>()
        .init_gizmo_group::<MapGizmos>()
        //RENDER
        .add_systems(Update, render)
        //RESOURCES
        .insert_resource(ViewInfo::default())
        .insert_resource(test_map())
        .insert_resource(Hits::no_hits())
        .insert_resource(PlayerCameraCache::default())
        .insert_resource(Vclip::full())
        .insert_resource(Light {
            direction: Vec3::new(-0.5, -0.5, 0.6).normalize(), // Light from top-left
            color: Color::WHITE,
            intensity: 1.5,
        })
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
    pub eye_height: f32,
    pub pitch: f32,
}

impl Default for ViewInfo {
    fn default() -> Self {
        let fov: f32 = 90.0;
        //Calculate the distance from the camera to the screen
        let view_distance = (WINDOW_WIDTH as f32) / 2.0 / (fov.to_radians() / 2.0).tan();
        let eye_height = 1.8;
        let pitch = 0.0;
        ViewInfo { fov, max_distance: 300.0, view_distance, eye_height, pitch }
    }
}

#[derive(Copy, Clone)]
pub struct VBounds {
    pub top: f32,
    pub bottom: f32,
}

#[derive(Resource)]
pub struct Vclip(pub Vec<VBounds>);

impl Vclip {
    pub fn full() -> Self {
        let mut vclip = Vclip(Vec::new());
        for _ in 0..WINDOW_WIDTH {
            vclip.0.push(VBounds::full());
        }
        vclip
    }
}

impl VBounds {
    pub fn full() -> Self {
        let (top, bottom) = ((WINDOW_HEIGHT as f32) / 2.0, -(WINDOW_HEIGHT as f32) / 2.0);
        VBounds { top: top, bottom: bottom }
    }
}

//Resource that syncs the player position to a resource
#[derive(Resource, Default)]
pub struct PlayerCameraCache {
    transform: Transform,
}

fn setup(mut commands: Commands) {
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
    commands.spawn((Player, Transform::from_xyz(50.0, 50.0, 0.0)));
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
pub fn test_map() -> Map {
    Map {
        sectors: vec![
            SectorBuilder::new(
                0,
                0.0,
                25.0,
                Color::srgb(1.0, 0.5, 1.0),
                Color::srgb(1.0, 0.0, 0.0),
                Some(vec![0])
            )
                .wall(0.0, 0.0, 100.0, 0.0, Color::srgb(1.0, 0.0, 0.0))
                .wall(100.0, 0.0, 100.0, 40.0, Color::srgb(0.0, 1.0, 0.0))
                .portal_with_steps(
                    100.0,
                    40.0,
                    100.0,
                    60.0,
                    1,
                    Some(Color::srgb(1.0, 1.0, 1.0)),
                    Some(Color::srgb(1.0, 1.0, 1.0))
                ) // back_sector only — front is auto = 0
                .wall(100.0, 60.0, 100.0, 100.0, Color::srgb(0.0, 1.0, 0.0))
                .wall(100.0, 100.0, 0.0, 100.0, Color::srgb(0.0, 0.0, 1.0))
                .wall(0.0, 100.0, 0.0, 0.0, Color::srgb(1.0, 1.0, 0.0))
                .build(),

            SectorBuilder::new(
                1,
                10.0,
                20.0,
                Color::srgb(1.0, 0.5, 1.0),
                Color::srgb(1.0, 0.5, 0.0),
                None
            )
                .wall(100.0, 40.0, 150.0, 40.0, Color::srgb(0.5, 0.5, 0.5))
                .portal_with_steps(
                    150.0,
                    40.0,
                    150.0,
                    60.0,
                    2,
                    Some(Color::srgb(1.0, 0.0, 0.5)),
                    Some(Color::srgb(1.0, 0.0, 0.5))
                )
                .wall(150.0, 60.0, 100.0, 60.0, Color::srgb(0.5, 0.5, 0.5))
                .portal_with_steps(
                    100.0,
                    60.0,
                    100.0,
                    40.0,
                    0,
                    Some(Color::srgb(1.0, 1.0, 0.0)),
                    Some(Color::srgb(1.0, 1.0, 1.0))
                )
                .build(),
            SectorBuilder::new(
                2,
                -10.0,
                25.0,
                Color::srgb(1.0, 0.5, 1.0),
                Color::srgb(1.0, 0.0, 0.0),
                None
            )
                .wall(150.0, 40.0, 150.0, 0.0, Color::srgb(1.0, 0.0, 0.0))
                .wall(150.0, 0.0, 250.0, 0.0, Color::srgb(1.0, 0.5, 0.0))
                .wall(250.0, 0.0, 250.0, 100.0, Color::srgb(0.7, 0.5, 0.0))
                .wall(250.0, 100.0, 150.0, 100.0, Color::srgb(0.7, 0.5, 1.0))
                .wall(150.0, 100.0, 150.0, 60.0, Color::srgb(0.7, 0.5, 1.0))
                .portal_with_steps(
                    150.0,
                    60.0,
                    150.0,
                    40.0,
                    1,
                    Some(Color::srgb(1.0, 0.0, 0.0)),
                    Some(Color::srgb(1.0, 1.0, 1.0))
                )
                .build()
        ],
        obstacle_sectors: vec![
            ObstacleSectorBuilder::new(
                0,
                0.0,
                5.0,
                Color::srgb(1.0, 0.0, 0.0),
                Color::srgb(0.0, 1.0, 1.0)
            )
                .wall(10.0, 10.0, 20.0, 10.0, Color::srgb(0.0, 1.0, 0.0))
                .wall(20.0, 10.0, 20.0, 20.0, Color::srgb(1.0, 0.0, 0.0))
                .wall(20.0, 20.0, 10.0, 20.0, Color::srgb(1.0, 1.0, 0.0))
                .wall(10.0, 20.0, 10.0, 10.0, Color::srgb(1.0, 0.0, 0.0))
                .build()
        ],
    }
}

//Bye bye
