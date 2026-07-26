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
            .insert_resource(WallEntityPool::default())
            .insert_resource(PortalBoundaryEntityPool::default())
            .insert_resource(VissEntityPool::default());
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
    mut wall_pool: ResMut<WallEntityPool>,
    mut portal_pool: ResMut<PortalBoundaryEntityPool>,
    mut viss_pool: ResMut<VissEntityPool>,
    mut query: Query<&mut Visibility>
) {
    let transform = &player_cache.transform;

    if let Some(player_sector_index) = find_player_sector(transform.translation.truncate(), &map) {
        let mut all_groups: Vec<WallGroup> = Vec::new();
        let mut portal_boundary_groups: Vec<PortalBoundaryGroup> = Vec::new();
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
            &mut visited_sectors
        );

        // Build viss groups from all visited sectors (one per unique sector)
        let viss_groups: Vec<VissGroup> = visited_sectors
            .iter()
            .map(|&sector_id| VissGroup {
                sector: map.sectors[sector_id].clone(),
            })
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
    }
}

fn recurse_sector(
    player_transform: &Transform,
    view_info: &ViewInfo,
    sector_index: usize,
    map: &Map,
    ray_origins: &[(usize, Vec2)],
    visited_per_ray: &mut HashMap<usize, HashSet<usize>>,
    all_groups: &mut Vec<WallGroup>,
    portal_boundary_groups: &mut Vec<PortalBoundaryGroup>,
    visited_sectors: &mut HashSet<usize>
) {
    // Track this sector as visited for floor/ceiling rendering
    visited_sectors.insert(sector_index);

    let mut hits: Vec<WallHit> = Vec::new();

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
            hits.push(hit);
        }
    }

    let grouped = group_hits_by_wall(hits);
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
                sector: front_sector.clone(),
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
            visited_sectors
        );
    }
}

// --------------WALL RENDERING---------------------
struct WallGroup {
    hits: Vec<WallHit>,
    wall: LineDef,
    sector: Sector,
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

    for i in 0..min(needed, pool_size) {
        let entity = pool.entities[i];
        let group = &groups[i];

        let mesh = build_wall_mesh(&group.hits, &group.wall, &group.sector);
        let material = StandardMaterial {
            base_color_texture: group.wall.front_side_def.textures.middle.clone(),
            ..default()
        };

        commands
            .entity(entity)
            .insert(Visibility::Visible)
            .insert(Mesh3d(meshes.add(mesh)))
            .insert(MeshMaterial3d(materials.add(material)));
    }

    for i in needed..pool.used.min(pool_size) {
        if let Ok(mut vis) = query.get_mut(pool.entities[i]) {
            *vis = Visibility::Hidden;
        }
    }

    if needed > pool_size {
        for i in pool_size..needed {
            let group = &groups[i];
            let entity = commands
                .spawn((
                    Visibility::Visible,
                    Mesh3d(meshes.add(build_wall_mesh(&group.hits, &group.wall, &group.sector))),
                    Transform::default(),
                ))
                .id();
            pool.entities.push(entity);
        }
    }

    pool.used = needed;
}

//--------------------------------PORTAL BOUNDARY RENDERING---------------------------
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
        let (upper_entity, lower_entity) = pool.entities[i];
        let group = &groups[i];

        if group.has_upper {
            let upper_mesh = build_portal_boundary_mesh(
                &group.hits,
                &group.wall,
                group.back_sector.ceiling_height,
                group.front_sector.ceiling_height
            );
            let upper_material = StandardMaterial {
                base_color_texture: group.wall.front_side_def.textures.upper.clone(),
                ..default()
            };
            commands
                .entity(upper_entity)
                .insert(Visibility::Visible)
                .insert(Mesh3d(meshes.add(upper_mesh)))
                .insert(MeshMaterial3d(materials.add(upper_material)));
        } else {
            if let Ok(mut vis) = query.get_mut(upper_entity) {
                *vis = Visibility::Hidden;
            }
        }

        if group.has_lower {
            let lower_mesh = build_portal_boundary_mesh(
                &group.hits,
                &group.wall,
                group.front_sector.floor_height,
                group.back_sector.floor_height
            );
            let lower_material = StandardMaterial {
                base_color_texture: group.wall.front_side_def.textures.lower.clone(),
                ..default()
            };
            commands
                .entity(lower_entity)
                .insert(Visibility::Visible)
                .insert(Mesh3d(meshes.add(lower_mesh)))
                .insert(MeshMaterial3d(materials.add(lower_material)));
        } else {
            if let Ok(mut vis) = query.get_mut(lower_entity) {
                *vis = Visibility::Hidden;
            }
        }
    }

    for i in needed..pool.used.min(pool_size) {
        let (upper_entity, lower_entity) = pool.entities[i];
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

            let upper_entity = if group.has_upper {
                let mesh = build_portal_boundary_mesh(
                    &group.hits,
                    &group.wall,
                    group.back_sector.ceiling_height,
                    group.front_sector.ceiling_height
                );
                let material = StandardMaterial {
                    base_color_texture: group.wall.front_side_def.textures.upper.clone(),
                    ..default()
                };
                commands
                    .spawn((
                        Visibility::Visible,
                        Mesh3d(meshes.add(mesh)),
                        MeshMaterial3d(materials.add(material)),
                        Transform::default(),
                    ))
                    .id()
            } else {
                commands.spawn((Visibility::Hidden, Transform::default())).id()
            };

            let lower_entity = if group.has_lower {
                let mesh = build_portal_boundary_mesh(
                    &group.hits,
                    &group.wall,
                    group.front_sector.floor_height,
                    group.back_sector.floor_height
                );
                let material = StandardMaterial {
                    base_color_texture: group.wall.front_side_def.textures.lower.clone(),
                    ..default()
                };
                commands
                    .spawn((
                        Visibility::Visible,
                        Mesh3d(meshes.add(mesh)),
                        MeshMaterial3d(materials.add(material)),
                        Transform::default(),
                    ))
                    .id()
            } else {
                commands.spawn((Visibility::Hidden, Transform::default())).id()
            };

            pool.entities.push((upper_entity, lower_entity));
        }
    }

    pool.used = needed;
}

//----------VISS PLANES RENDERING------------------

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

    // 1. Activate/reuse pool entities
    for i in 0..min(needed, pool_size) {
        let (ceil_entity, floor_entity) = pool.entities[i];
        let group = &groups[i];
        let sector = &group.sector;

        let ceil_mesh = build_viss_mesh(sector, sector.ceiling_height, false);
        let ceil_material = StandardMaterial {
            base_color_texture: Some(sector.ceiling_texture.clone()),
            ..default()
        };
        commands
            .entity(ceil_entity)
            .insert(Visibility::Visible)
            .insert(Mesh3d(meshes.add(ceil_mesh)))
            .insert(MeshMaterial3d(materials.add(ceil_material)));

        let floor_mesh = build_viss_mesh(sector, sector.floor_height, true);
        let floor_material = StandardMaterial {
            base_color_texture: Some(sector.floor_texture.clone()),
            ..default()
        };
        commands
            .entity(floor_entity)
            .insert(Visibility::Visible)
            .insert(Mesh3d(meshes.add(floor_mesh)))
            .insert(MeshMaterial3d(materials.add(floor_material)));
    }

    // 2. Hide unused entities
    for i in needed..pool.used.min(pool_size) {
        let (ceil_entity, floor_entity) = pool.entities[i];
        if let Ok(mut vis) = query.get_mut(ceil_entity) {
            *vis = Visibility::Hidden;
        }
        if let Ok(mut vis) = query.get_mut(floor_entity) {
            *vis = Visibility::Hidden;
        }
    }

    // 3. Spawn overflow
    if needed > pool_size {
        for i in pool_size..needed {
            let group = &groups[i];
            let sector = &group.sector;

            let ceil_mesh = build_viss_mesh(sector, sector.ceiling_height, false);
            let ceil_material = StandardMaterial {
                base_color_texture: Some(sector.ceiling_texture.clone()),
                ..default()
            };
            let ceil_entity = commands
                .spawn((
                    Visibility::Visible,
                    Mesh3d(meshes.add(ceil_mesh)),
                    MeshMaterial3d(materials.add(ceil_material)),
                    Transform::default(),
                ))
                .id();

            let floor_mesh = build_viss_mesh(sector, sector.floor_height, true);
            let floor_material = StandardMaterial {
                base_color_texture: Some(sector.floor_texture.clone()),
                ..default()
            };
            let floor_entity = commands
                .spawn((
                    Visibility::Visible,
                    Mesh3d(meshes.add(floor_mesh)),
                    MeshMaterial3d(materials.add(floor_material)),
                    Transform::default(),
                ))
                .id();

            pool.entities.push((ceil_entity, floor_entity));
        }
    }

    pool.used = needed;
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
            entities: Vec::with_capacity(64),
            used: 0,
        }
    }
}

#[derive(Resource)]
pub struct PortalBoundaryEntityPool {
    pub entities: Vec<(Entity, Entity)>,
    pub used: usize,
}

impl Default for PortalBoundaryEntityPool {
    fn default() -> Self {
        Self {
            entities: Vec::with_capacity(64),
            used: 0,
        }
    }
}

#[derive(Resource)]
pub struct VissEntityPool {
    pub entities: Vec<(Entity, Entity)>,
    pub used: usize,
}

impl Default for VissEntityPool {
    fn default() -> Self {
        Self {
            entities: Vec::with_capacity(64),
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

    let p0 = start.pos;
    let p1 = end.pos;

    let wall_length = wall.start.distance(wall.end);
    let u0 = p0.distance(wall.start) / wall_length;
    let u1 = p1.distance(wall.start) / wall_length;

    let positions = vec![
        [p0.x, p0.y, sector.floor_height],
        [p1.x, p1.y, sector.floor_height],
        [p1.x, p1.y, sector.ceiling_height],
        [p0.x, p0.y, sector.ceiling_height]
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
            let v_coord = if range_y > 0.0 { (v.y - min_y) / range_y } else { 0.0 };
            [u, v_coord]
        })
        .collect();

    let indices = triangulate_polygon(&vertices, facing_up);

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
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

fn wall_normal(line_def: &LineDef) -> Vec2 {
    let dir = (line_def.end - line_def.start).normalize_or_zero();
    Vec2::new(dir.y, -dir.x)
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
