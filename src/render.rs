use std::cmp::min;

use bevy::{ mesh::PrimitiveTopology, prelude::* };
use crate::ray::*;
use crate::map::*;
use crate::*;

//------------------PLUGIN---------------------------------------
pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render)
            .add_systems(Update, render_2d)
            .insert_resource(WallEntityPool::default());
    }
}
//------------------MAIN RENDER FUNCTIONS------------------------
pub fn render_2d(
    mut gizmos: Gizmos<MapGizmos>,
    map: Res<Map>,
    view_info: Res<ViewInfo>,
    transform_query: Query<&Transform, With<Player>>
) {
    let transform = transform_query.single().unwrap();
    for i in 0..RAY_COUNT {
        if let Some(sector) = find_player_sector(transform.translation.truncate(), &map) {
            let sector = &map.sectors[sector];
            if let Some(hit) = get_hit_sector(&transform, &view_info, sector.id, &map, i) {
                let x = hit_to_screen_x(&view_info, i);
                let window_top = project_height(
                    map.sectors[sector.id].ceiling_height - EYE_OFFSET,
                    hit.perp_dist,
                    &view_info
                );
                let window_bottom = project_height(
                    map.sectors[sector.id].floor_height - EYE_OFFSET,
                    hit.perp_dist,
                    &view_info
                );
                gizmos.line_2d(Vec2::new(x, window_top), Vec2::new(x, window_bottom), Color::WHITE);
            }
        }
    }
}

pub fn render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_cache: Res<PlayerCameraCache>,
    map: Res<Map>,
    view_info: Res<ViewInfo>,
    mut pool: ResMut<WallEntityPool>,
    mut query: Query<&mut Visibility>
) {
    let transform = &player_cache.transform;
    // Find the sector the player is in
    if let Some(player_sector_index) = find_player_sector(transform.translation.truncate(), &map) {
        let mut hits = Vec::new();
        //1. HIT GROUPING
        for i in 0..RAY_COUNT {
            //Get hits for the sector the player is in
            let hit = get_hit_sector(&transform, &view_info, player_sector_index, &map, i).unwrap();
            hits.push(hit);
        }
        let grouped_hits = group_hits_by_wall(hits);

        //2. POOL MANAGEMENT
        let needed = grouped_hits.len();
        let pool_size = pool.entities.len();
        //TODO GET ENTITY INFO
        for i in 0..min(needed, pool_size) {
            let entity = pool.entities[i];
            let hit_group = &grouped_hits[i];
            if !hit_group[0].is_portal {
                let id = hit_group[0].wall_id;
                let wall_index = id.index; // let mesh = build_wall_mesh(hit_group, sector);
                let sector_index = id.sector;
                let sector = &map.sectors[sector_index];
                let wall = &sector.walls[wall_index];
                let mesh = build_wall_mesh(&hit_group, &wall, &sector);
                let material = StandardMaterial {
                    base_color_texture: map.sectors[sector_index].walls[
                        wall_index
                    ].front_side_def.textures.middle.clone(),
                    ..default()
                };
                commands
                    .entity(entity)
                    .insert(Visibility::Visible)
                    .insert(Mesh3d(meshes.add(mesh)))
                    .insert(MeshMaterial3d(materials.add(material)));
            }
        }
        //3. HIDE NOT NEEDED ENTITIES
        for i in needed..pool.used.min(pool_size) {
            if let Ok(mut vis) = query.get_mut(pool.entities[i]) {
                *vis = Visibility::Hidden;
            }
        }

        //4. SPAWN OVERFLOW
        if needed > pool_size {
            for i in pool_size..needed {
                let hit_group = &grouped_hits[i];
                let id = hit_group[0].wall_id;
                let wall_index = id.index; // let mesh = build_wall_mesh(hit_group, sector);
                let sector_index = id.sector;
                let sector = &map.sectors[sector_index];
                let wall = &sector.walls[wall_index];

                let entity = commands
                    .spawn((
                        Visibility::Visible,
                        Mesh3d(meshes.add(build_wall_mesh(hit_group, &wall, &sector))),
                        // MeshMaterial2d(materials.add(build_wall_material(hit_group, sector))),
                        Transform::default(),
                    ))
                    .id();

                pool.entities.push(entity);
            }
        }
    }
}
//-------------------------------RESOURCES-------------------------------

#[derive(Resource)]
pub struct WallEntityPool {
    pub entities: Vec<Entity>,
    pub used: usize,
}

impl Default for WallEntityPool {
    fn default() -> Self {
        Self {
            entities: Vec::with_capacity(64), // pre-allocate
            used: 0,
        }
    }
}

// ------------------------------RENDER HELPERS------------------------------
fn project_height(world_height: f32, dist: f32, view_info: &ViewInfo) -> f32 {
    let relative = world_height - view_info.eye_height;
    (relative * view_info.view_distance) / dist + view_info.pitch
}

pub fn build_wall_mesh(hit_group: &[WallHit], wall: &LineDef, sector: &Sector) -> Mesh {
    let start = hit_group.first().unwrap();
    let end = hit_group.last().unwrap();

    // World-space positions (where the rays actually hit the wall)
    let p0 = start.pos; // left hit point on wall
    let p1 = end.pos; // right hit point on wall

    // Wall length in world space for texture scaling
    let wall_length = wall.start.distance(wall.end);

    // Distance along the wall from its start point
    let u0 = p0.distance(wall.start) / wall_length;
    let u1 = p1.distance(wall.start) / wall_length;

    // Build mesh with world-space positions
    let positions = vec![
        [p0.x, p0.y, sector.floor_height],
        [p1.x, p1.y, sector.floor_height],
        [p1.x, p1.y, sector.ceiling_height],
        [p0.x, p0.y, sector.ceiling_height]
    ];

    let normal = wall_normal(wall).extend(0.0);
    let normals = vec![[normal.x, normal.y, normal.z]; 4];

    // UVs anchored to world-space position on the wall
    let uvs = vec![
        [u0, 1.0], // bottom-left
        [u1, 1.0], // bottom-right
        [u1, 0.0], // top-right
        [u0, 0.0] // top-left
    ];

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(vec![0, 2, 1, 0, 3, 2]))
}

fn wall_normal(line_def: &LineDef) -> Vec2 {
    let dir = (line_def.end - line_def.start).normalize_or_zero();
    Vec2::new(dir.y, -dir.x) // inward normal for CCW sectors
}

fn group_hits_by_wall(hits: Vec<WallHit>) -> Vec<Vec<WallHit>> {
    let mut grouped_hits: Vec<Vec<WallHit>> = Vec::new();
    let mut current_group: Vec<WallHit> = Vec::new();

    for hit in hits {
        if current_group.is_empty() {
            current_group.push(hit);
        } else {
            let last_hit = current_group.last().unwrap();
            if last_hit.wall_id == hit.wall_id && last_hit.sector_id == hit.sector_id {
                current_group.push(hit);
            } else {
                grouped_hits.push(current_group);
                current_group = vec![hit];
            }
        }
    }

    if !current_group.is_empty() {
        grouped_hits.push(current_group);
    }

    grouped_hits
}
