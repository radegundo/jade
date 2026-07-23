use bevy::{
    camera::{ RenderTarget, visibility::RenderLayers },
    prelude::*,
    window::{ PresentMode, WindowRef, WindowResolution },
};
use bevy_grid::*;

//Screen width
//For naming purposes duplicate the constant
const WINDOW_WIDTH: usize = 1920;
const WINDOW_HEIGHT: u32 = 1080;

const EYE_OFFSET: f32 = 1.6;

const RAY_COUNT: usize = WINDOW_WIDTH;

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

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    //Spawn Map Window
    let resolution: WindowResolution = (500, 500).into();
    let window_size = Vec2::new(resolution.width(), resolution.height());

    let map_win = commands
        .spawn((Window { resolution: resolution, resizable: false, ..default() }, MapWindowMarker))
        .id();

    //Spawn Map Camera
    commands.spawn((
        Camera2d,
        RenderLayers::layer(1),
        RenderTarget::Window(WindowRef::Entity(map_win)),
    ));

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

#[derive(Component)]
struct MapWindowMarker;
