use bevy::{
    camera::{ Viewport, visibility::RenderLayers },
    prelude::*,
    window::{ PresentMode, WindowResolution },
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
        .insert_resource(ViewInfo::default())
        .insert_resource(PlayerCameraCache::default())
        .insert_resource(FpsState::default())
        .add_systems(Update, sync_camera_to_player)
        .add_systems(Update, update_player_cache)
        .add_systems(Update, update_fps)
        .add_systems(Update, toggle_fps_visible)
        .run();
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct FpsCounterMarker;

#[derive(Resource)]
pub struct FpsState {
    pub visible: bool,
    frame_count: u32,
    elapsed: f32,
    display_value: f32,
}

impl Default for FpsState {
    fn default() -> Self {
        Self { visible: true, frame_count: 0, elapsed: 0.0, display_value: 0.0 }
    }
}

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

    let minimap_size = UVec2::new(250, 250);
    let margin = 20;
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            viewport: Some(Viewport {
                physical_position: UVec2::new(1920 - minimap_size.x - margin, margin),
                physical_size: minimap_size,
                ..default()
            }),
            ..default()
        },
        RenderLayers::layer(1),
    ));

    commands.spawn((
        Text::new("FPS: --"),
        TextFont::from_font_size(18.0),
        TextColor(Color::srgb(0.3, 1.0, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        FpsCounterMarker,
    ));

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

fn update_fps(
    time: Res<Time>,
    mut state: ResMut<FpsState>,
    mut query: Query<&mut Text, With<FpsCounterMarker>>,
) {
    state.frame_count += 1;
    state.elapsed += time.delta_secs();
    if state.elapsed >= 0.4 {
        state.display_value = state.frame_count as f32 / state.elapsed;
        state.frame_count = 0;
        state.elapsed = 0.0;
        if let Ok(mut text) = query.single_mut() {
            text.0 = format!("FPS: {:.1}", state.display_value);
        }
    }
}

fn toggle_fps_visible(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<FpsState>,
    mut query: Query<&mut Visibility, With<FpsCounterMarker>>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        state.visible = !state.visible;
        if let Ok(mut vis) = query.single_mut() {
            *vis = if state.visible { Visibility::Visible } else { Visibility::Hidden };
        }
    }
}
