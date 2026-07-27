use bevy::{
    prelude::*,
    window::{ PresentMode, WindowResolution },
    dev_tools::fps_overlay::FpsOverlayPlugin,
};

use crate::{ input::OwnInputPlugin, map::MapPlugin, render::RenderPlugin };

mod ray;
mod map;
mod render;
mod systems;
mod input;

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
        .add_systems(Startup, setup)
        .add_plugins(RenderPlugin)
        .add_plugins(MapPlugin)
        .add_plugins(OwnInputPlugin)
        .add_plugins(FpsOverlayPlugin::default())
        .add_systems(Update, sync_camera_to_player)
        .insert_resource(ViewInfo::default())
        .insert_resource(PlayerCameraCache::default())
        .add_systems(Update, update_player_cache)
        .run();
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct MapWindowMarker;

#[derive(Resource)]
pub struct ViewInfo {
    pub fov: f32,
    pub max_distance: f32,
    pub view_distance: f32,
    pub eye_height: f32,
    pub pitch: f32,
}

impl Default for ViewInfo {
    fn default() -> Self {
        let fov: f32 = 90.0;
        let view_distance = (WINDOW_WIDTH as f32) / 2.0 / (fov.to_radians() / 2.0).tan();
        let eye_height = 1.8;
        let pitch = 0.0;
        ViewInfo { fov, max_distance: 300.0, view_distance, eye_height, pitch }
    }
}

#[derive(Resource, Default)]
pub struct PlayerCameraCache {
    pub transform: Transform,
}

fn update_player_cache(
    mut player_cache: ResMut<PlayerCameraCache>,
    transform_query: Query<&Transform, With<Player>>
) {
    let transform = transform_query.single().unwrap();
    player_cache.transform = *transform;
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera3d::default());

    let resolution: WindowResolution = (1920, 1080).into();
    let _window_size = Vec2::new(resolution.width(), resolution.height());

    // let map_win = commands
    //     .spawn((Window { resolution: resolution, resizable: false, ..default() }, MapWindowMarker))
    //     .id();

    // commands.spawn((
    //     Camera2d,
    //     RenderLayers::layer(1),
    //     RenderTarget::Window(WindowRef::Entity(map_win)),
    // ));

    commands.spawn((Player, Transform::from_xyz(50.0, 50.0, 0.0)));
}

fn sync_camera_to_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
    view_info: Res<ViewInfo>
) {
    if let (Ok(player), Ok(mut camera)) = (player_query.single(), camera_query.single_mut()) {
        let pos = player.translation;
        let angle = player.rotation.to_euler(EulerRot::XYZ).2;

        camera.translation = Vec3::new(pos.x, pos.y, view_info.eye_height);

        let look_target = Vec3::new(
            pos.x + angle.cos(),
            pos.y + angle.sin(),
            view_info.eye_height + view_info.pitch
        );
        camera.look_at(look_target, Vec3::Z);
    }
}
