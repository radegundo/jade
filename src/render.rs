use std::cmp::min;

use bevy::platform::collections::{ HashMap, HashSet };
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
            .add_systems(PostStartup, spawn_obstacle_entities)
            .insert_resource(WallEntityPool::default())
            .insert_resource(PortalBoundaryEntityPool::default())
            .insert_resource(VissEntityPool::default())
            .insert_resource(ObstacleEntities::default());
    }
}

//------------------OBSTACLE STARTUP SPAWNING--------------------

/// Uniquely identifies one edge of one obstacle in one sector.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ObstacleEdgeKey {
    sector_id: usize,
    obstacle_id: usize,
    edge_index: usize,
}

/// Resource that holds the pre-spawned entity for every obstacle edge.
/// Built once at PostStartup, never modified.
#[derive(Resource, Default)]
pub(crate) struct ObstacleEntities {
    pub(crate) edges: HashMap<ObstacleEdgeKey, Entity>,
}

/// Runs once after startup.
/// For every obstacle edge in the map, builds the full mesh and spawns
/// a Hidden entity. Each entity is stored by ObstacleEdgeKey.
/// The render system only toggles Visibility — no mesh work at runtime.
fn spawn_obstacle_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<Map>,
    mut obstacle_entities: ResMut<ObstacleEntities>
) {
    for sector in &map.sectors {
        for obstacle in &sector.obstacles {
            for (edge_index, edge) in obstacle.edges.iter().enumerate() {
                let mesh = build_obstacle_edge_mesh(edge, obstacle.bottom, obstacle.top);
                let material = StandardMaterial {
                    base_color_texture: Some(obstacle.texture.clone()),
                    ..default()
                };

                let entity = commands
                    .spawn((
                        Visibility::Hidden,
                        Mesh3d(meshes.add(mesh)),
                        MeshMaterial3d(materials.add(material)),
                        Transform::default(),
                    ))
                    .id();

                obstacle_entities.edges.insert(
                    ObstacleEdgeKey { sector_id: sector.id, obstacle_id: obstacle.id, edge_index },
                    entity
                );
            }
        }
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
        if let Some(sector_idx) = find_player_sector(transform.translation.truncate(), &map) {
            let sector = &map.sectors[sector_idx];
            if let Some(hit) = get_hit_sector(&transform, &view_info, sector.id, &map, i) {
                let x = hit_to_screen_x(&view_info, i);
                let window_top = project_height(
                    sector.ceiling_height - EYE_OFFSET,
                    hit.perp_dist,
                    &view_info
                );
                let window_bottom = project_height(
                    sector.floor_height - EYE_OFFSET,
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
    mut wall_pool: ResMut<WallEntityPool>,
    mut portal_pool: ResMut<PortalBoundaryEntityPool>,
    mut viss_pool: ResMut<VissEntityPool>,
    obstacle_entities: Res<ObstacleEntities>,
    mut query: Query<&mut Visibility>
) {
    let transform = &player_cache.transform;

    if let Some(player_sector_index) = find_player_sector(transform.translation.truncate(), &map) {
        let mut all_groups: Vec<WallGroup> = Vec::new();
        let mut portal_boundary_groups: Vec<PortalBoundaryGroup> = Vec::new();
        let mut visible_obstacles: HashSet<ObstacleEdgeKey> = HashSet::new();
        let mut visited_sectors: HashSet<usize> = HashSet::new();

        let initial_origins: Vec<(usize, Vec2)> = (0..RAY_COUNT)
            .map(|i| (i, transform.translation.truncate()))
            .collect();

        let mut visited_per_ray: HashMap<usize, HashSet<usize>> = HashMap::new();

        recurse_sector(
            transform,
            &view_info,
            player_sector_index,
            &map,
            &initial_origins,
            &mut visited_per_ray,
            &mut all_groups,
            &mut portal_boundary_groups,
            &mut visible_obstacles,
            &mut visited_sectors
        );

        let viss_groups: Vec<VissGroup> = visited_sectors
            .iter()
            .map(|&sector_id| VissGroup { sector: map.sectors[sector_id].clone() })
            .collect();

        render_wall_groups(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut wall_pool,
            &mut query,
            &all_groups
        );
        render_portal_boundary_groups(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut portal_pool,
            &mut query,
            &portal_boundary_groups
        );
        render_viss_groups(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut viss_pool,
            &mut query,
            &viss_groups
        );

        // Toggle obstacle visibility — no mesh work, just Visible/Hidden
        for (key, &entity) in obstacle_entities.edges.iter() {
            if let Ok(mut vis) = query.get_mut(entity) {
                *vis = if visible_obstacles.contains(key) {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

//------------------SECTOR RECURSION-----------------------------

fn recurse_sector(
    player_transform: &Transform,
    view_info: &ViewInfo,
    sector_index: usize,
    map: &Map,
    ray_origins: &[(usize, Vec2)],
    visited_per_ray: &mut HashMap<usize, HashSet<usize>>,
    all_groups: &mut Vec<WallGroup>,
    portal_boundary_groups: &mut Vec<PortalBoundaryGroup>,
    visible_obstacles: &mut HashSet<ObstacleEdgeKey>,
    visited_sectors: &mut HashSet<usize>
) {
    visited_sectors.insert(sector_index);

    let mut wall_hits: Vec<WallHit> = Vec::new();

    for &(index, origin) in ray_origins {
        let visited = visited_per_ray.entry(index).or_default();
        if visited.contains(&sector_index) {
            continue;
        }
        visited.insert(sector_index);

        let angle = get_ray_angle(index, player_transform, view_info);
        let offset = get_ray_offset(index, view_info);

        if
            let Some(hit) = get_hit_sector_recursive(
                origin,
                angle,
                offset,
                view_info,
                sector_index,
                map,
                index
            )
        {
            let max_dist_sq = origin.distance_squared(hit.pos);

            // Check each obstacle — if any ray reaches it, mark its edges visible
            let sector = &map.sectors[sector_index];
            for obstacle in &sector.obstacles {
                if
                    ray_hits_obstacle(
                        origin,
                        angle,
                        view_info,
                        sector_index,
                        map,
                        max_dist_sq,
                        obstacle.id
                    )
                {
                    // Mark every visible edge of this obstacle
                    let end = origin + Vec2::new(angle.cos(), angle.sin()) * view_info.max_distance;
                    let ray = make_ray(origin, end);
                    for (edge_index, edge) in obstacle.edges.iter().enumerate() {
                        if test_ray_hit(&ray, edge, origin, max_dist_sq) {
                            visible_obstacles.insert(ObstacleEdgeKey {
                                sector_id: sector_index,
                                obstacle_id: obstacle.id,
                                edge_index,
                            });
                        }
                    }
                }
            }

            wall_hits.push(hit);
        }
    }

    // Group consecutive wall hits by wall_id
    let grouped = group_hits_by_wall(wall_hits);
    let mut portal_next: HashMap<usize, Vec<(usize, Vec2)>> = HashMap::new();

    for group in grouped {
        if group.is_empty() {
            continue;
        }

        let front_sector = &map.sectors[sector_index];
        let wall = front_sector.walls[group[0].wall_id.index].clone();

        if group[0].is_portal {
            if let Some(back_sector_id) = group[0].back_sector {
                let back_sector = &map.sectors[back_sector_id];

                let has_lower = back_sector.floor_height > front_sector.floor_height;
                let has_upper = back_sector.ceiling_height < front_sector.ceiling_height;

                if has_lower || has_upper {
                    portal_boundary_groups.push(PortalBoundaryGroup {
                        hits: group.clone(),
                        wall: wall.clone(),
                        front_sector: front_sector.clone(),
                        back_sector: back_sector.clone(),
                        has_upper,
                        has_lower,
                    });
                }

                for hit in &group {
                    let angle = get_ray_angle(hit.ray_index, player_transform, view_info);
                    let dir = Vec2::new(angle.cos(), angle.sin());
                    let nudged = hit.pos + dir * 0.05;
                    portal_next.entry(back_sector_id).or_default().push((hit.ray_index, nudged));
                }
            }
        } else {
            all_groups.push(WallGroup {
                hits: group,
                wall,
            });
        }
    }

    for (next_sector, origins) in portal_next {
        recurse_sector(
            player_transform,
            view_info,
            next_sector,
            map,
            &origins,
            visited_per_ray,
            all_groups,
            portal_boundary_groups,
            visible_obstacles,
            visited_sectors
        );
    }
}

//------------------WALL RENDERING---------------------

struct WallGroup {
    hits: Vec<WallHit>,
    wall: LineDef,
}

fn render_wall_groups(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    pool: &mut ResMut<WallEntityPool>,
    query: &mut Query<&mut Visibility>,
    groups: &[WallGroup]
) {
    let needed = groups.len();
    let pool_size = pool.entities.len();

    // Reuse existing pool slots — overwrite mesh data in place
    for i in 0..min(needed, pool_size) {
        let (entity, ref mesh_handle) = pool.entities[i];
        let group = &groups[i];

        if let Some(mut existing) = meshes.get_mut(mesh_handle) {
            *existing = build_wall_mesh(&group.hits, &group.wall);
        }

        commands
            .entity(entity)
            .insert(Visibility::Visible)
            .insert(
                MeshMaterial3d(
                    materials.add(StandardMaterial {
                        base_color_texture: group.wall.front_side_def.textures.middle.clone(),
                        ..default()
                    })
                )
            );
    }

    // Hide unused pool slots
    for i in needed..pool.used.min(pool_size) {
        let (entity, _) = pool.entities[i];
        if let Ok(mut vis) = query.get_mut(entity) {
            *vis = Visibility::Hidden;
        }
    }

    // Spawn new entities if pool is not large enough
    if needed > pool_size {
        for i in pool_size..needed {
            let group = &groups[i];
            let mesh_handle = meshes.add(build_wall_mesh(&group.hits, &group.wall));
            let entity = commands
                .spawn((
                    Visibility::Visible,
                    Mesh3d(mesh_handle.clone()),
                    MeshMaterial3d(
                        materials.add(StandardMaterial {
                            base_color_texture: group.wall.front_side_def.textures.middle.clone(),
                            ..default()
                        })
                    ),
                    Transform::default(),
                ))
                .id();
            pool.entities.push((entity, mesh_handle));
        }
    }

    pool.used = needed;
}

//------------------PORTAL BOUNDARY RENDERING---------------------------

struct PortalBoundaryGroup {
    hits: Vec<WallHit>,
    wall: LineDef,
    front_sector: Sector,
    back_sector: Sector,
    has_upper: bool,
    has_lower: bool,
}

fn render_portal_boundary_groups(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    pool: &mut ResMut<PortalBoundaryEntityPool>,
    query: &mut Query<&mut Visibility>,
    groups: &[PortalBoundaryGroup]
) {
    let needed = groups.len();
    let pool_size = pool.entities.len();

    for i in 0..min(needed, pool_size) {
        let (upper_entity, ref upper_mesh, lower_entity, ref lower_mesh) = pool.entities[i];
        let group = &groups[i];

        if group.has_upper {
            if let Some(mut existing) = meshes.get_mut(upper_mesh) {
                *existing = build_portal_boundary_mesh(
                    &group.hits,
                    &group.wall,
                    group.back_sector.ceiling_height,
                    group.front_sector.ceiling_height
                );
            }
            commands
                .entity(upper_entity)
                .insert(Visibility::Visible)
                .insert(
                    MeshMaterial3d(
                        materials.add(StandardMaterial {
                            base_color_texture: group.wall.front_side_def.textures.upper.clone(),
                            ..default()
                        })
                    )
                );
        } else {
            if let Ok(mut vis) = query.get_mut(upper_entity) {
                *vis = Visibility::Hidden;
            }
        }

        if group.has_lower {
            if let Some(mut existing) = meshes.get_mut(lower_mesh) {
                *existing = build_portal_boundary_mesh(
                    &group.hits,
                    &group.wall,
                    group.front_sector.floor_height,
                    group.back_sector.floor_height
                );
            }
            commands
                .entity(lower_entity)
                .insert(Visibility::Visible)
                .insert(
                    MeshMaterial3d(
                        materials.add(StandardMaterial {
                            base_color_texture: group.wall.front_side_def.textures.lower.clone(),
                            ..default()
                        })
                    )
                );
        } else {
            if let Ok(mut vis) = query.get_mut(lower_entity) {
                *vis = Visibility::Hidden;
            }
        }
    }

    for i in needed..pool.used.min(pool_size) {
        let (upper_entity, _, lower_entity, _) = pool.entities[i];
        if let Ok(mut vis) = query.get_mut(upper_entity) {
            *vis = Visibility::Hidden;
        }
        if let Ok(mut vis) = query.get_mut(lower_entity) {
            *vis = Visibility::Hidden;
        }
    }

    if needed > pool_size {
        for i in pool_size..needed {
            let group = &groups[i];

            let upper_mesh_handle = meshes.add(
                if group.has_upper {
                    build_portal_boundary_mesh(
                        &group.hits,
                        &group.wall,
                        group.back_sector.ceiling_height,
                        group.front_sector.ceiling_height
                    )
                } else {
                    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
                }
            );
            let upper_entity = commands
                .spawn((
                    if group.has_upper { Visibility::Visible } else { Visibility::Hidden },
                    Mesh3d(upper_mesh_handle.clone()),
                    MeshMaterial3d(
                        materials.add(StandardMaterial {
                            base_color_texture: group.wall.front_side_def.textures.upper.clone(),
                            ..default()
                        })
                    ),
                    Transform::default(),
                ))
                .id();

            let lower_mesh_handle = meshes.add(
                if group.has_lower {
                    build_portal_boundary_mesh(
                        &group.hits,
                        &group.wall,
                        group.front_sector.floor_height,
                        group.back_sector.floor_height
                    )
                } else {
                    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
                }
            );
            let lower_entity = commands
                .spawn((
                    if group.has_lower { Visibility::Visible } else { Visibility::Hidden },
                    Mesh3d(lower_mesh_handle.clone()),
                    MeshMaterial3d(
                        materials.add(StandardMaterial {
                            base_color_texture: group.wall.front_side_def.textures.lower.clone(),
                            ..default()
                        })
                    ),
                    Transform::default(),
                ))
                .id();

            pool.entities.push((upper_entity, upper_mesh_handle, lower_entity, lower_mesh_handle));
        }
    }

    pool.used = needed;
}

//------------------VISS PLANES (FLOORS AND CEILINGS)------------------

struct VissGroup {
    sector: Sector,
}

fn render_viss_groups(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    pool: &mut ResMut<VissEntityPool>,
    query: &mut Query<&mut Visibility>,
    groups: &[VissGroup]
) {
    let needed = groups.len();
    let pool_size = pool.entities.len();

    for i in 0..min(needed, pool_size) {
        let (ceil_entity, ref ceil_mesh, floor_entity, ref floor_mesh) = pool.entities[i];
        let sector = &groups[i].sector;

        if let Some(mut existing) = meshes.get_mut(ceil_mesh) {
            *existing = build_viss_mesh(sector, sector.ceiling_height, false);
        }
        commands
            .entity(ceil_entity)
            .insert(Visibility::Visible)
            .insert(
                MeshMaterial3d(
                    materials.add(StandardMaterial {
                        base_color_texture: Some(sector.ceiling_texture.clone()),
                        ..default()
                    })
                )
            );

        if let Some(mut existing) = meshes.get_mut(floor_mesh) {
            *existing = build_viss_mesh(sector, sector.floor_height, true);
        }
        commands
            .entity(floor_entity)
            .insert(Visibility::Visible)
            .insert(
                MeshMaterial3d(
                    materials.add(StandardMaterial {
                        base_color_texture: Some(sector.floor_texture.clone()),
                        ..default()
                    })
                )
            );
    }

    for i in needed..pool.used.min(pool_size) {
        let (ceil_entity, _, floor_entity, _) = pool.entities[i];
        if let Ok(mut vis) = query.get_mut(ceil_entity) {
            *vis = Visibility::Hidden;
        }
        if let Ok(mut vis) = query.get_mut(floor_entity) {
            *vis = Visibility::Hidden;
        }
    }

    if needed > pool_size {
        for i in pool_size..needed {
            let sector = &groups[i].sector;

            let ceil_mesh_handle = meshes.add(
                build_viss_mesh(sector, sector.ceiling_height, false)
            );
            let ceil_entity = commands
                .spawn((
                    Visibility::Visible,
                    Mesh3d(ceil_mesh_handle.clone()),
                    MeshMaterial3d(
                        materials.add(StandardMaterial {
                            base_color_texture: Some(sector.ceiling_texture.clone()),
                            ..default()
                        })
                    ),
                    Transform::default(),
                ))
                .id();

            let floor_mesh_handle = meshes.add(build_viss_mesh(sector, sector.floor_height, true));
            let floor_entity = commands
                .spawn((
                    Visibility::Visible,
                    Mesh3d(floor_mesh_handle.clone()),
                    MeshMaterial3d(
                        materials.add(StandardMaterial {
                            base_color_texture: Some(sector.floor_texture.clone()),
                            ..default()
                        })
                    ),
                    Transform::default(),
                ))
                .id();

            pool.entities.push((ceil_entity, ceil_mesh_handle, floor_entity, floor_mesh_handle));
        }
    }

    pool.used = needed;
}

//------------------RESOURCES-------------------------------

#[derive(Resource)]
pub struct WallEntityPool {
    /// (entity, mesh_handle) — mesh_handle lets us overwrite in place
    pub entities: Vec<(Entity, Handle<Mesh>)>,
    pub used: usize,
}

impl Default for WallEntityPool {
    fn default() -> Self {
        Self { entities: Vec::with_capacity(64), used: 0 }
    }
}

#[derive(Resource)]
pub struct PortalBoundaryEntityPool {
    /// (upper_entity, upper_mesh, lower_entity, lower_mesh)
    pub entities: Vec<(Entity, Handle<Mesh>, Entity, Handle<Mesh>)>,
    pub used: usize,
}

impl Default for PortalBoundaryEntityPool {
    fn default() -> Self {
        Self { entities: Vec::with_capacity(64), used: 0 }
    }
}

#[derive(Resource)]
pub struct VissEntityPool {
    /// (ceil_entity, ceil_mesh, floor_entity, floor_mesh)
    pub entities: Vec<(Entity, Handle<Mesh>, Entity, Handle<Mesh>)>,
    pub used: usize,
}

impl Default for VissEntityPool {
    fn default() -> Self {
        Self { entities: Vec::with_capacity(64), used: 0 }
    }
}

//------------------MESH BUILDERS------------------------------

fn project_height(world_height: f32, dist: f32, view_info: &ViewInfo) -> f32 {
    let relative = world_height - view_info.eye_height;
    (relative * view_info.view_distance) / dist + view_info.pitch
}

/// Wall mesh is built from ray hit positions (partial wall slice).
/// bottom/top come from the hit itself so the same function works
/// for walls at any height.
pub fn build_wall_mesh(hit_group: &[WallHit], wall: &LineDef) -> Mesh {
    let start = hit_group.first().unwrap();
    let end = hit_group.last().unwrap();

    let p0 = start.pos;
    let p1 = end.pos;

    let wall_length = wall.start.distance(wall.end);
    let u0 = p0.distance(wall.start) / wall_length;
    let u1 = p1.distance(wall.start) / wall_length;

    let positions = vec![
        [p0.x, p0.y, start.bottom],
        [p1.x, p1.y, start.bottom],
        [p1.x, p1.y, start.top],
        [p0.x, p0.y, start.top]
    ];

    let normal = wall_normal(wall).extend(0.0);
    let normals = vec![[normal.x, normal.y, normal.z]; 4];
    let uvs = vec![[u0, 1.0], [u1, 1.0], [u1, 0.0], [u0, 0.0]];

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(vec![0, 2, 1, 0, 3, 2]))
}

/// Portal boundary mesh fills the height gap between two sectors.
/// floor_height/ceiling_height are passed explicitly because the
/// boundary height range doesn't come from either sector alone.
pub fn build_portal_boundary_mesh(
    hit_group: &[WallHit],
    wall: &LineDef,
    floor_height: f32,
    ceiling_height: f32
) -> Mesh {
    let start = hit_group.first().unwrap();
    let end = hit_group.last().unwrap();

    let p0 = start.pos;
    let p1 = end.pos;

    let wall_length = wall.start.distance(wall.end);
    let u0 = p0.distance(wall.start) / wall_length;
    let u1 = p1.distance(wall.start) / wall_length;

    let positions = vec![
        [p0.x, p0.y, floor_height],
        [p1.x, p1.y, floor_height],
        [p1.x, p1.y, ceiling_height],
        [p0.x, p0.y, ceiling_height]
    ];

    let normal = wall_normal(wall).extend(0.0);
    let normals = vec![[normal.x, normal.y, normal.z]; 4];
    let uvs = vec![[u0, 1.0], [u1, 1.0], [u1, 0.0], [u0, 0.0]];

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(vec![0, 2, 1, 0, 3, 2]))
}

/// Obstacle mesh is built from the full LineDef geometry (not ray hits).
/// Built once at startup and never rebuilt.
/// UVs span 0..1 across the full edge length.
pub fn build_obstacle_edge_mesh(edge: &LineDef, bottom: f32, top: f32) -> Mesh {
    let p0 = edge.start;
    let p1 = edge.end;

    let positions = vec![
        [p0.x, p0.y, bottom],
        [p1.x, p1.y, bottom],
        [p1.x, p1.y, top],
        [p0.x, p0.y, top]
    ];

    let normal = wall_normal(edge).extend(0.0);
    let normals = vec![[normal.x, normal.y, normal.z]; 4];
    let uvs = vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(vec![0, 2, 1, 0, 3, 2]))
}

/// Viss mesh triangulates the sector polygon at a given height.
/// facing_up controls normal direction and triangle winding.
pub fn build_viss_mesh(sector: &Sector, height: f32, facing_up: bool) -> Mesh {
    let vertices: Vec<Vec2> = sector.walls
        .iter()
        .map(|wall| wall.start)
        .collect();

    let vertex_count = vertices.len();
    if vertex_count < 3 {
        return Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    }

    let positions: Vec<[f32; 3]> = vertices
        .iter()
        .map(|v| [v.x, v.y, height])
        .collect();
    let normal = if facing_up { [0.0, 0.0, 1.0] } else { [0.0, 0.0, -1.0] };
    let normals: Vec<[f32; 3]> = vec![normal; vertex_count];

    let min_x = vertices
        .iter()
        .map(|v| v.x)
        .fold(f32::INFINITY, f32::min);
    let max_x = vertices
        .iter()
        .map(|v| v.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = vertices
        .iter()
        .map(|v| v.y)
        .fold(f32::INFINITY, f32::min);
    let max_y = vertices
        .iter()
        .map(|v| v.y)
        .fold(f32::NEG_INFINITY, f32::max);
    let range_x = max_x - min_x;
    let range_y = max_y - min_y;

    let uvs: Vec<[f32; 2]> = vertices
        .iter()
        .map(|v| {
            let u = if range_x > 0.0 { (v.x - min_x) / range_x } else { 0.0 };
            let vc = if range_y > 0.0 { (v.y - min_y) / range_y } else { 0.0 };
            [u, vc]
        })
        .collect();

    let indices = triangulate_polygon(&vertices, facing_up);

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

//------------------GEOMETRY HELPERS------------------------------

fn wall_normal(line_def: &LineDef) -> Vec2 {
    let dir = (line_def.end - line_def.start).normalize_or_zero();
    Vec2::new(dir.y, -dir.x)
}

fn triangulate_polygon(vertices: &[Vec2], facing_up: bool) -> Vec<u32> {
    let n = vertices.len();
    if n < 3 {
        return vec![];
    }

    let mut remaining: Vec<usize> = (0..n).collect();
    let mut indices: Vec<u32> = Vec::with_capacity((n - 2) * 3);

    let signed_area: f32 =
        remaining
            .windows(2)
            .map(|w| {
                let a = vertices[w[0]];
                let b = vertices[w[1]];
                (b.x - a.x) * (b.y + a.y)
            })
            .sum::<f32>() +
        ({
            let a = vertices[*remaining.last().unwrap()];
            let b = vertices[remaining[0]];
            (b.x - a.x) * (b.y + a.y)
        });

    let is_ccw = signed_area < 0.0;
    let mut iterations = 0;
    let max_iterations = n * n;

    while remaining.len() > 2 && iterations < max_iterations {
        iterations += 1;
        let len = remaining.len();
        let mut ear_found = false;

        for i in 0..len {
            let prev = remaining[(i + len - 1) % len];
            let curr = remaining[i];
            let next = remaining[(i + 1) % len];

            let a = vertices[prev];
            let b = vertices[curr];
            let c = vertices[next];

            let cross = (b - a).perp_dot(c - b);
            let is_convex = if is_ccw { cross > 0.0 } else { cross < 0.0 };
            if !is_convex {
                continue;
            }

            let mut contains_point = false;
            for j in 0..len {
                let idx = remaining[j];
                if idx == prev || idx == curr || idx == next {
                    continue;
                }
                if point_in_triangle(vertices[idx], a, b, c) {
                    contains_point = true;
                    break;
                }
            }
            if contains_point {
                continue;
            }

            if facing_up {
                if is_ccw {
                    indices.push(prev as u32);
                    indices.push(curr as u32);
                    indices.push(next as u32);
                } else {
                    indices.push(next as u32);
                    indices.push(curr as u32);
                    indices.push(prev as u32);
                }
            } else {
                if is_ccw {
                    indices.push(next as u32);
                    indices.push(curr as u32);
                    indices.push(prev as u32);
                } else {
                    indices.push(prev as u32);
                    indices.push(curr as u32);
                    indices.push(next as u32);
                }
            }

            remaining.remove(i);
            ear_found = true;
            break;
        }

        if !ear_found {
            break;
        }
    }

    indices
}

fn point_in_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let v0 = c - a;
    let v1 = b - a;
    let v2 = p - a;

    let dot00 = v0.dot(v0);
    let dot01 = v0.dot(v1);
    let dot02 = v0.dot(v2);
    let dot11 = v1.dot(v1);
    let dot12 = v1.dot(v2);

    let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

    u >= 0.0 && v >= 0.0 && u + v <= 1.0
}

fn group_hits_by_wall(hits: Vec<WallHit>) -> Vec<Vec<WallHit>> {
    let mut grouped: Vec<Vec<WallHit>> = Vec::new();
    let mut current: Vec<WallHit> = Vec::new();

    for hit in hits {
        if current.is_empty() {
            current.push(hit);
        } else {
            let last = current.last().unwrap();
            if last.wall_id == hit.wall_id && last.sector_id == hit.sector_id {
                current.push(hit);
            } else {
                grouped.push(current);
                current = vec![hit];
            }
        }
    }

    if !current.is_empty() {
        grouped.push(current);
    }

    grouped
}
