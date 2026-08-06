use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
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
            .add_systems(PostStartup, (spawn_obstacle_entities, spawn_viss_entities, spawn_wall_entities))
            .insert_resource(VissEntities::default())
            .insert_resource(ObstacleEntities::default())
            .insert_resource(WallEntities::default())
            .insert_resource(MaterialCache::default());
    }
}

//------------------OBSTACLE STARTUP SPAWNING--------------------

/// Identifies one renderable surface of an obstacle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ObstacleSurface {
    /// One of the vertical side edges, identified by index in obstacle.edges
    Side(usize),
    /// The horizontal top cap
    Top,
    /// The horizontal bottom cap
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ObstacleEdgeKey {
    pub(crate) sector_id: usize,
    pub(crate) obstacle_id: usize,
    pub(crate) surface: ObstacleSurface,
}

// Resource that holds the pre-spawned entity for every obstacle edge.
// Built once at PostStartup, never modified.
#[derive(Resource, Default)]
pub(crate) struct ObstacleEntities {
    pub(crate) edges: HashMap<ObstacleEdgeKey, Entity>,
}

// Runs once after startup.
// For every obstacle edge in the map, builds the full mesh and spawns
// a Hidden entity. Each entity is stored by ObstacleEdgeKey.
// The render system only toggles Visibility — no mesh work at runtime.
fn spawn_obstacle_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<Map>,
    mut obstacle_entities: ResMut<ObstacleEntities>
) {
    for sector in &map.sectors {
        for obstacle in &sector.obstacles {
            // --- SIDE EDGES (one entity per edge, exactly as before) ---
            for (edge_index, edge) in obstacle.edges.iter().enumerate() {
                let mesh = build_obstacle_edge_mesh(edge, obstacle.bottom, obstacle.top, &map.vertices);
                let material = StandardMaterial {
                    base_color_texture: Some(obstacle.side_texture.clone()),
                    cull_mode: None,
                    double_sided: true,
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
                    ObstacleEdgeKey {
                        sector_id: sector.id,
                        obstacle_id: obstacle.id,
                        surface: ObstacleSurface::Side(edge_index),
                    },
                    entity
                );
            }

            // --- TOP CAP ---
            let top_mesh = build_obstacle_cap_mesh(&obstacle.edges, obstacle.top, true, &map.vertices);
            let top_material = StandardMaterial {
                base_color_texture: Some(obstacle.top_texture.clone()),
                cull_mode: None,
                double_sided: true,
                ..default()
            };
            let top_entity = commands
                .spawn((
                    Visibility::Hidden,
                    Mesh3d(meshes.add(top_mesh)),
                    MeshMaterial3d(materials.add(top_material)),
                    Transform::default(),
                ))
                .id();
            obstacle_entities.edges.insert(
                ObstacleEdgeKey {
                    sector_id: sector.id,
                    obstacle_id: obstacle.id,
                    surface: ObstacleSurface::Top,
                },
                top_entity
            );

            // --- BOTTOM CAP ---
            let bottom_mesh = build_obstacle_cap_mesh(&obstacle.edges, obstacle.bottom, false, &map.vertices);
            let bottom_material = StandardMaterial {
                base_color_texture: Some(obstacle.bottom_texture.clone()),
                cull_mode: None,
                double_sided: true,
                ..default()
            };
            let bottom_entity = commands
                .spawn((
                    Visibility::Hidden,
                    Mesh3d(meshes.add(bottom_mesh)),
                    MeshMaterial3d(materials.add(bottom_material)),
                    Transform::default(),
                ))
                .id();
            obstacle_entities.edges.insert(
                ObstacleEdgeKey {
                    sector_id: sector.id,
                    obstacle_id: obstacle.id,
                    surface: ObstacleSurface::Bottom,
                },
                bottom_entity
            );
        }
    }
}

//------------------WALL STARTUP SPAWNING--------------------

/// One renderable surface of a wall.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum WallSurface {
    /// A solid wall, spanning its sector's full floor..ceiling.
    Solid,
    /// The upper step frame of a portal (gap between two ceiling heights).
    Upper,
    /// The lower step frame of a portal (gap between two floor heights).
    Lower,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct WallKey {
    pub(crate) sector_id: usize,
    pub(crate) wall_index: usize,
    pub(crate) surface: WallSurface,
}

// Resource that holds the pre-spawned entity for every wall.
// Built once at PostStartup, never modified.
#[derive(Resource, Default)]
pub(crate) struct WallEntities {
    pub(crate) by_key: HashMap<WallKey, Entity>,
}

// Runs once after startup. For every wall in the map, builds the full mesh and
// spawns a Hidden entity. The render system only toggles Visibility.
fn spawn_wall_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<Map>,
    mut wall_entities: ResMut<WallEntities>,
    mut material_cache: ResMut<MaterialCache>
) {
    // Each portal appears in both sectors' wall lists (same vertices, opposite
    // direction). Dedupe by the unordered vertex pair so the step frame is
    // only spawned once.
    let mut spawned_portals: HashSet<(usize, usize)> = HashSet::new();

    for sector in &map.sectors {
        for (wall_index, wall) in sector.walls.iter().enumerate() {
            let key = |surface| WallKey {
                sector_id: sector.id,
                wall_index,
                surface,
            };

            let (lo, hi) = (wall.start_idx.min(wall.end_idx), wall.start_idx.max(wall.end_idx));

            match &wall.back_side_def {
                None => {
                    let mesh = build_obstacle_edge_mesh(
                        wall,
                        sector.floor_height,
                        sector.ceiling_height,
                        &map.vertices
                    );
                    let mat = material_cache.get_or_create(
                        &mut materials,
                        wall.front_side_def.textures.middle.clone()
                    );
                    let entity = commands
                        .spawn((
                            Visibility::Hidden,
                            Mesh3d(meshes.add(mesh)),
                            MeshMaterial3d(mat),
                            Transform::default(),
                        ))
                        .id();
                    wall_entities.by_key.insert(key(WallSurface::Solid), entity);
                }
                Some(back) => {
                    if !spawned_portals.insert((lo, hi)) {
                        continue;
                    }
                    let back_sector = &map.sectors[back.facing];
                    let back_sector_id = back.facing;

                    // Find the matching wall index in the back sector so the step
                    // frame can be keyed under both sectors — it's visible from
                    // either side of the portal.
                    let back_wall_index = back_sector.walls.iter().position(|w| {
                        let (blo, bhi) = (w.start_idx.min(w.end_idx), w.start_idx.max(w.end_idx));
                        (blo, bhi) == (lo, hi)
                    });

                    let mut register = |surface: WallSurface, entity: Entity| {
                        wall_entities.by_key.insert(
                            WallKey { sector_id: sector.id, wall_index, surface },
                            entity
                        );
                        if let Some(bwi) = back_wall_index {
                            wall_entities.by_key.insert(
                                WallKey { sector_id: back_sector_id, wall_index: bwi, surface },
                                entity
                            );
                        }
                    };

                    // Upper step frame spans between the two ceiling heights.
                    if (sector.ceiling_height - back_sector.ceiling_height).abs() > 0.001 {
                        let bottom = sector.ceiling_height.min(back_sector.ceiling_height);
                        let top = sector.ceiling_height.max(back_sector.ceiling_height);
                        let mesh = build_obstacle_edge_mesh(wall, bottom, top, &map.vertices);
                        let mat = material_cache.get_or_create(
                            &mut materials,
                            wall.front_side_def.textures.upper.clone()
                        );
                        let entity = commands
                            .spawn((
                                Visibility::Hidden,
                                Mesh3d(meshes.add(mesh)),
                                MeshMaterial3d(mat),
                                Transform::default(),
                            ))
                            .id();
                        register(WallSurface::Upper, entity);
                    }

                    // Lower step frame spans between the two floor heights.
                    if (sector.floor_height - back_sector.floor_height).abs() > 0.001 {
                        let bottom = sector.floor_height.min(back_sector.floor_height);
                        let top = sector.floor_height.max(back_sector.floor_height);
                        let mesh = build_obstacle_edge_mesh(wall, bottom, top, &map.vertices);
                        let mat = material_cache.get_or_create(
                            &mut materials,
                            wall.front_side_def.textures.lower.clone()
                        );
                        let entity = commands
                            .spawn((
                                Visibility::Hidden,
                                Mesh3d(meshes.add(mesh)),
                                MeshMaterial3d(mat),
                                Transform::default(),
                            ))
                            .id();
                        register(WallSurface::Lower, entity);
                    }
                }
            }
        }
    }
}

//------------------VISS STARTUP SPAWNING--------------------

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum VissSurface {
    Floor,
    Ceiling,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct VissKey {
    pub(crate) sector_id: usize,
    pub(crate) surface: VissSurface,
}

// Resource that holds the pre-spawned floor/ceiling entity for every sector.
// Built once at PostStartup, never modified.
#[derive(Resource, Default)]
pub(crate) struct VissEntities {
    pub(crate) by_key: HashMap<VissKey, Entity>,
}

// Runs once after startup. Floor/ceiling geometry is static, so meshes and
// materials are built once. The render system only toggles Visibility.
fn spawn_viss_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<Map>,
    mut viss_entities: ResMut<VissEntities>,
    mut material_cache: ResMut<MaterialCache>
) {
    for sector in &map.sectors {
        let floor_mesh = build_viss_mesh(sector, sector.floor_height, true, &map.vertices);
        let floor_mat = material_cache.get_or_create(
            &mut materials,
            Some(sector.floor_texture.clone())
        );
        let floor_entity = commands
            .spawn((
                Visibility::Hidden,
                Mesh3d(meshes.add(floor_mesh)),
                MeshMaterial3d(floor_mat),
                Transform::default(),
            ))
            .id();
        viss_entities.by_key.insert(
            VissKey { sector_id: sector.id, surface: VissSurface::Floor },
            floor_entity
        );

        let ceil_mesh = build_viss_mesh(sector, sector.ceiling_height, false, &map.vertices);
        let ceil_mat = material_cache.get_or_create(
            &mut materials,
            Some(sector.ceiling_texture.clone())
        );
        let ceil_entity = commands
            .spawn((
                Visibility::Hidden,
                Mesh3d(meshes.add(ceil_mesh)),
                MeshMaterial3d(ceil_mat),
                Transform::default(),
            ))
            .id();
        viss_entities.by_key.insert(
            VissKey { sector_id: sector.id, surface: VissSurface::Ceiling },
            ceil_entity
        );
    }
}

//------------------MAIN RENDER FUNCTIONS------------------------

//LEGACY 2D RENDER
pub fn _render_2d(
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
                let window_top = _project_height(
                    sector.ceiling_height - EYE_OFFSET,
                    hit.perp_dist,
                    &view_info
                );
                let window_bottom = _project_height(
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
    player_cache: Res<PlayerCameraCache>,
    map: Res<Map>,
    view_info: Res<ViewInfo>,
    viss_entities: Res<VissEntities>,
    obstacle_entities: Res<ObstacleEntities>,
    wall_entities: Res<WallEntities>,
    mut query: Query<&mut Visibility>
) {
    let transform = &player_cache.transform;
    let vertices = &map.vertices;

    if let Some(player_sector_index) = find_player_sector(transform.translation.truncate(), &map) {
        let ray_table = build_ray_table(transform, &view_info);

        let mut visible_obstacles: HashSet<ObstacleEdgeKey> = HashSet::new();
        let mut visited_sectors: HashSet<usize> = HashSet::new();
        let mut wall_visible_sectors: HashSet<usize> = HashSet::new();

        let initial_origins: Vec<(usize, Vec2)> = (0..RAY_COUNT)
            .map(|i| (i, transform.translation.truncate()))
            .collect();

        let mut visited_per_ray: HashMap<usize, HashSet<usize>> = HashMap::new();

        recurse_sector(
            &view_info,
            &ray_table,
            player_sector_index,
            &map,
            vertices,
            &initial_origins,
            &mut visited_per_ray,
            &mut visible_obstacles,
            &mut visited_sectors,
            &mut wall_visible_sectors
        );

        // Toggle floor/ceiling visibility — no mesh work, just Visible/Hidden
        for (key, &entity) in viss_entities.by_key.iter() {
            if let Ok(mut vis) = query.get_mut(entity) {
                *vis = if visited_sectors.contains(&key.sector_id) {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }

        // Toggle wall visibility — solid walls and portal step frames
        for (key, &entity) in wall_entities.by_key.iter() {
            if let Ok(mut vis) = query.get_mut(entity) {
                *vis = if wall_visible_sectors.contains(&key.sector_id) {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }

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

//------------------RAY TABLE-----------------------------

struct RayTable {
    offsets: Vec<f32>,
    dirs: Vec<Vec2>,
}

// Precomputes angle/offset/direction for every ray once per frame.
// Matches get_ray_angle/get_ray_offset arithmetic exactly.
fn build_ray_table(transform: &Transform, view_info: &ViewInfo) -> RayTable {
    let player_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
    let fov_rad = effective_fov(view_info).to_radians();
    let half_fov = fov_rad / 2.0;
    let angle_step = fov_rad / ((RAY_COUNT as f32) - 1.0).max(1.0);

    let offsets: Vec<f32> = (0..RAY_COUNT)
        .map(|i| -half_fov + angle_step * (i as f32))
        .collect();
    let dirs: Vec<Vec2> = (0..RAY_COUNT)
        .map(|i| {
            let angle = player_angle - half_fov + angle_step * (i as f32);
            Vec2::new(angle.cos(), angle.sin())
        })
        .collect();

    RayTable { offsets, dirs }
}

//------------------SECTOR RECURSION-----------------------------

fn recurse_sector(
    view_info: &ViewInfo,
    ray_table: &RayTable,
    sector_index: usize,
    map: &Map,
    vertices: &[Vec2],
    ray_origins: &[(usize, Vec2)],
    visited_per_ray: &mut HashMap<usize, HashSet<usize>>,
    visible_obstacles: &mut HashSet<ObstacleEdgeKey>,
    visited_sectors: &mut HashSet<usize>,
    wall_visible_sectors: &mut HashSet<usize>
) {
    visited_sectors.insert(sector_index);

    let mut portal_next: HashMap<usize, Vec<(usize, Vec2)>> = HashMap::new();

    for &(index, origin) in ray_origins {
        let visited = visited_per_ray.entry(index).or_default();
        if visited.contains(&sector_index) {
            continue;
        }
        visited.insert(sector_index);

        let dir = ray_table.dirs[index];
        let offset = ray_table.offsets[index];
        let end = origin + dir * view_info.max_distance;

        if
            let Some(hit) = get_hit_sector_recursive(
                origin,
                dir,
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
                        end,
                        sector_index,
                        map,
                        max_dist_sq,
                        obstacle.id
                    )
                {
                    let ray = make_ray(origin, end);

                    let mut any_side_visible = false;

                    for (edge_index, edge) in obstacle.edges.iter().enumerate() {
                        if test_ray_hit(&ray, edge, origin, max_dist_sq, vertices) {
                            visible_obstacles.insert(ObstacleEdgeKey {
                                sector_id: sector_index,
                                obstacle_id: obstacle.id,
                                surface: ObstacleSurface::Side(edge_index),
                            });
                            any_side_visible = true;
                        }
                    }

                    // Top and bottom caps are visible whenever any side is visible
                    if any_side_visible {
                        visible_obstacles.insert(ObstacleEdgeKey {
                            sector_id: sector_index,
                            obstacle_id: obstacle.id,
                            surface: ObstacleSurface::Top,
                        });
                        visible_obstacles.insert(ObstacleEdgeKey {
                            sector_id: sector_index,
                            obstacle_id: obstacle.id,
                            surface: ObstacleSurface::Bottom,
                        });
                    }
                }
            }

            // Every wall hit (portal or solid) marks its owning sector visible,
            // toggling that sector's walls. For a portal the owner sector's
            // step frames need this even if no solid wall of that sector is
            // directly hit.
            wall_visible_sectors.insert(hit.wall_id.sector);

            if hit.is_portal {
                if let Some(back_sector_id) = hit.back_sector {
                    let dir = ray_table.dirs[hit.ray_index];
                    let nudged = hit.pos + dir * 0.05;
                    portal_next.entry(back_sector_id).or_default().push((hit.ray_index, nudged));
                }
            }
        }
    }

    for (next_sector, origins) in portal_next {
        recurse_sector(
            view_info,
            ray_table,
            next_sector,
            map,
            vertices,
            &origins,
            visited_per_ray,
            visible_obstacles,
            visited_sectors,
            wall_visible_sectors
        );
    }
}

//------------------WALL RENDERING---------------------

//------------------RESOURCES-------------------------------

// Caches StandardMaterials by texture handle so surfaces don't allocate
// a fresh material asset every frame. Materials depend only on a texture.
#[derive(Resource, Default)]
pub struct MaterialCache {
    by_texture: HashMap<Option<Handle<Image>>, Handle<StandardMaterial>>,
}

impl MaterialCache {
    pub fn get_or_create(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
        texture: Option<Handle<Image>>
    ) -> Handle<StandardMaterial> {
        self.by_texture
            .entry(texture.clone())
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color_texture: texture,
                    cull_mode: None,
                    double_sided: true,
                    ..default()
                })
            })
            .clone()
    }
}

//------------------MESH BUILDERS------------------------------

//LEGACY HEIGHT PROJECTION
fn _project_height(world_height: f32, dist: f32, view_info: &ViewInfo) -> f32 {
    let relative = world_height - view_info.eye_height;
    (relative * view_info.view_distance) / dist + view_info.pitch
}

// Obstacle mesh is built from the full LineDef geometry (not ray hits).
// Built once at startup and never rebuilt.
// UVs span 0..1 across the full edge length.
pub fn build_obstacle_edge_mesh(edge: &LineDef, bottom: f32, top: f32, vertices: &[Vec2]) -> Mesh {
    let p0 = *edge.start(vertices);
    let p1 = *edge.end(vertices);

    let positions = vec![
        [p0.x, p0.y, bottom],
        [p1.x, p1.y, bottom],
        [p1.x, p1.y, top],
        [p0.x, p0.y, top]
    ];

    let normal = wall_normal(edge, vertices).extend(0.0);
    let normals = vec![[normal.x, normal.y, normal.z]; 4];
    let uvs = vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(vec![0, 2, 1, 0, 3, 2]))
}

// Viss mesh triangulates the sector polygon at a given height.
// facing_up controls normal direction and triangle winding.
pub fn build_viss_mesh(sector: &Sector, height: f32, facing_up: bool, vertices: &[Vec2]) -> Mesh {
    let verts: Vec<Vec2> = sector.walls
        .iter()
        .map(|wall| *wall.start(vertices))
        .collect();

    let vertex_count = verts.len();
    if vertex_count < 3 {
        return Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    }

    let positions: Vec<[f32; 3]> = verts
        .iter()
        .map(|v| [v.x, v.y, height])
        .collect();
    let normal = if facing_up { [0.0, 0.0, 1.0] } else { [0.0, 0.0, -1.0] };
    let normals: Vec<[f32; 3]> = vec![normal; vertex_count];

    let min_x = verts
        .iter()
        .map(|v| v.x)
        .fold(f32::INFINITY, f32::min);
    let max_x = verts
        .iter()
        .map(|v| v.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = verts
        .iter()
        .map(|v| v.y)
        .fold(f32::INFINITY, f32::min);
    let max_y = verts
        .iter()
        .map(|v| v.y)
        .fold(f32::NEG_INFINITY, f32::max);
    let range_x = max_x - min_x;
    let range_y = max_y - min_y;

    let uvs: Vec<[f32; 2]> = verts
        .iter()
        .map(|v| {
            let u = if range_x > 0.0 { (v.x - min_x) / range_x } else { 0.0 };
            let vc = if range_y > 0.0 { (v.y - min_y) / range_y } else { 0.0 };
            [u, vc]
        })
        .collect();

    let indices = triangulate_polygon(&verts, facing_up);

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

/// Builds a horizontal cap mesh (top or bottom face) for an obstacle.
/// Uses the obstacle edge start points to form the polygon outline,
/// then triangulates it exactly like a viss plane.
/// facing_up: true = top cap (normal +Z), false = bottom cap (normal -Z)
pub fn build_obstacle_cap_mesh(edges: &[LineDef], height: f32, facing_up: bool, vertices: &[Vec2]) -> Mesh {
    // Collect polygon vertices from edge start points
    let verts: Vec<Vec2> = edges
        .iter()
        .map(|e| *e.start(vertices))
        .collect();

    let vertex_count = verts.len();
    if vertex_count < 3 {
        return Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    }

    let positions: Vec<[f32; 3]> = verts
        .iter()
        .map(|v| [v.x, v.y, height])
        .collect();

    let normal = if facing_up { [0.0, 0.0, 1.0] } else { [0.0, 0.0, -1.0] };
    let normals: Vec<[f32; 3]> = vec![normal; vertex_count];

    // Normalize UVs to the bounding box of the obstacle footprint
    let min_x = verts
        .iter()
        .map(|v| v.x)
        .fold(f32::INFINITY, f32::min);
    let max_x = verts
        .iter()
        .map(|v| v.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = verts
        .iter()
        .map(|v| v.y)
        .fold(f32::INFINITY, f32::min);
    let max_y = verts
        .iter()
        .map(|v| v.y)
        .fold(f32::NEG_INFINITY, f32::max);
    let range_x = max_x - min_x;
    let range_y = max_y - min_y;

    let uvs: Vec<[f32; 2]> = verts
        .iter()
        .map(|v| {
            let u = if range_x > 0.0 { (v.x - min_x) / range_x } else { 0.0 };
            let vc = if range_y > 0.0 { (v.y - min_y) / range_y } else { 0.0 };
            [u, vc]
        })
        .collect();

    // Obstacle edges are CW, so the polygon winding is CW.
    // Pass facing_up so triangulate_polygon emits correct winding for each face.
    let indices = triangulate_polygon(&verts, facing_up);

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

//------------------GEOMETRY HELPERS------------------------------

fn wall_normal(line_def: &LineDef, vertices: &[Vec2]) -> Vec2 {
    let start = *line_def.start(vertices);
    let end = *line_def.end(vertices);
    let dir = (end - start).normalize_or_zero();
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

