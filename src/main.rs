use bevy::{
    asset::RenderAssetUsages,
    camera::{ RenderTarget, visibility::RenderLayers },
    mesh::Indices,
    prelude::*,
    window::{ PresentMode, WindowRef, WindowResolution },
};

use crate::{ input::OwnInputPlugin, map::{ relative_map::RelativeMapPlugin, * } };
use crate::ray::*;
use crate::render::*;

mod ray;
mod map;
mod render;
mod systems;
mod input;

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
        //-------------------------SETUP--------------------------
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_map)
        //-------------------------RENDER--------------------------
        .add_systems(Update, render_2d)
        //---------------------------MAP--------------------------
        .add_systems(Startup, setup_gizmo_layers)
        .init_gizmo_group::<MapGizmos>()
        .add_plugins(RelativeMapPlugin)
        //--------------------------INPUT--------------------------
        .add_plugins(OwnInputPlugin)
        .add_systems(Update, sync_camera_to_player)
        //--------------------------RESOURCES--------------------------
        .insert_resource(ViewInfo::default())
        .insert_resource(PlayerCameraCache::default())
        .add_systems(Update, update_player_cache)
        //--------------------------TEST--------------------------
        .add_systems(Startup, test_wall_render)
        .run();
}

//Player marker
#[derive(Component)]
struct Player;

//Marker for the map window
#[derive(Component)]
struct MapWindowMarker;

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

#[derive(Resource, Default)]
struct PlayerCameraCache {
    pub transform: Transform,
}

fn update_player_cache(
    mut player_cache: ResMut<PlayerCameraCache>,
    transform_query: Query<&Transform, With<Player>>
) {
    let transform = transform_query.single().unwrap();
    player_cache.transform = *transform;
}

//-----------------------------SETUP FUNCTIONS--------------------------------
fn setup(mut commands: Commands) {
    commands.spawn(Camera3d::default());

    //Spawn Map Window
    let resolution: WindowResolution = (1920, 1080).into();
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
    //Spawn player
    commands.spawn((Player, Transform::from_xyz(-10.0, 0.0, 0.0)));
}

//-----------------------------GIZMO CONFIGS--------------------------------
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct MapGizmos;

fn setup_gizmo_layers(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<MapGizmos>();
    config.render_layers = RenderLayers::layer(1);
}

//-----------------------------MAP SETUP--------------------------------
fn setup_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(test_map(asset_server));
}
//-----------------------------SYNC--------------------------------
fn sync_camera_to_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
    view_info: Res<ViewInfo>
) {
    if let (Ok(player), Ok(mut camera)) = (player_query.single(), camera_query.single_mut()) {
        let pos = player.translation;
        let angle = player.rotation.to_euler(EulerRot::XYZ).2;

        // Position camera at player position + eye height
        camera.translation = Vec3::new(pos.x, pos.y, view_info.eye_height);

        // Look in the direction the player is facing (XY plane)
        //Note: It is needed to invert the x-axis because the camera looks in the negative z direction by default
        let look_target = Vec3::new(pos.x - angle.cos(), pos.y + angle.sin(), view_info.eye_height);
        camera.look_at(look_target, Vec3::Z);
    }
}

//-----------------------------TEST---------------------------------
fn test_wall_render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>
) {
    let texture: Handle<Image> = asset_server.load("texture.png");
    let mesh = Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [0.0, 0.0, 0.0], // bottom-left
                [0.0, 0.0, 100.0], // top-left
                [100.0, 0.0, 100.0], // top-right
                [100.0, 0.0, 0.0] // bottom-right
            ]
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0]]
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
        )
        .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(
            materials.add(StandardMaterial {
                base_color_texture: Some(texture),
                ..default()
            })
        ),
        Transform::default(),
    ));
}
